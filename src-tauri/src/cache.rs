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
                aliases: Vec::new(),
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

    // MIG-004 §9: include the full alias table in the graph snapshot.
    // Sub-millisecond on the reference universe (~1.4k rows).
    let t_alias = Instant::now();
    let aliases = read_aliases(&conn)?;
    timings.push(("read_aliases".into(), t_alias.elapsed().as_millis() as u64));

    let server_return_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Ok(BootSnapshotGraph { links, tags, aliases, timings_ms: timings, server_return_unix_ms, server_start_unix_ms })
}

fn read_aliases(conn: &Connection) -> Result<Vec<NoteAliasOut>, String> {
    let mut stmt = conn
        .prepare("SELECT path, alias_lower FROM note_aliases")
        .map_err(|e| format!("prepare aliases: {}", e))?;
    let rows = stmt
        .query_map([], |row| Ok(NoteAliasOut {
            path: row.get(0)?,
            alias_lower: row.get(1)?,
        }))
        .map_err(|e| format!("query aliases: {}", e))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("row aliases: {}", e))?);
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

/// Sky View snapshot from the persisted sky_* tables. Linear in rows,
/// no JS-side iteration, no IPC re-serialization of raw note_links.
#[tauri::command]
pub fn cache_boot_snapshot_sky(app: tauri::AppHandle) -> Result<BootSnapshotSky, String> {
    let mut timings: Vec<(String, u64)> = Vec::new();

    let t0 = Instant::now();
    let _ = crate::search::ensure_search_db_ready(&app);
    timings.push(("ensure_db".into(), t0.elapsed().as_millis() as u64));

    let t1 = Instant::now();
    let conn = open_reader(&app)?;
    timings.push(("open_reader".into(), t1.elapsed().as_millis() as u64));

    // Readiness gate. If the back-fill hasn't stamped yet, return empty
    // + is_ready=false so the frontend falls back to the legacy path.
    let stored_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'sky'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let is_ready = stored_version >= crate::search::SKY_SCHEMA_VERSION;
    if !is_ready {
        return Ok(BootSnapshotSky {
            nodes: Vec::new(),
            links: Vec::new(),
            is_ready,
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

    let t2 = Instant::now();
    // Scan 1: nodes, flat.
    let mut nodes = read_sky_nodes_raw(&conn)?;
    timings.push(("scan_nodes".into(), t2.elapsed().as_millis() as u64));

    // Build O(1) lookup maps from the already-loaded node list:
    //   - path → index (for source resolution in links)
    //   - name (raw case) → index (for incoming count accumulation)
    // Same memory, no extra DB work.
    let mut path_to_idx: std::collections::HashMap<String, usize> =
        std::collections::HashMap::with_capacity(nodes.len());
    let mut name_to_idx: std::collections::HashMap<String, usize> =
        std::collections::HashMap::with_capacity(nodes.len());
    for (i, n) in nodes.iter().enumerate() {
        path_to_idx.insert(n.path.clone(), i);
        name_to_idx.insert(n.name.clone(), i);
    }

    // MIG-004 §8: load the alias resolution map. alias_lower → path so
    // an inbound link targeting an aliased name still resolves to the
    // renamed note's current row. ~1.4k entries on the reference
    // universe (~80 KB heap). One scan, no per-row queries.
    let t_alias = Instant::now();
    let mut alias_to_path: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT alias_lower, path FROM note_aliases")
            .map_err(|e| format!("prepare aliases: {}", e))?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| format!("query aliases: {}", e))?;
        for r in rows {
            let (alias, path) = r.map_err(|e| format!("row aliases: {}", e))?;
            // First insert wins on collision — matches the FS resolver's
            // path-sort tiebreak intent (deterministic; can refine later
            // if it needs to match `find_note_by_name_or_alias` exactly).
            alias_to_path.entry(alias).or_insert(path);
        }
    }
    timings.push(("scan_aliases".into(), t_alias.elapsed().as_millis() as u64));

    let t3 = Instant::now();
    // Scan 2: links, flat — no JOIN. Same loop resolves source
    // (path → id) and accumulates both link_count (incoming, by target
    // name) and outgoing_count (by source path). One pass = one set of
    // counts + the final link list ready for serialization.
    //
    // MIG-004 §8: when target_name doesn't match any current note name,
    // try the alias map before giving up. Fixes the rename-drops-edges
    // symptom in the Sky View boot payload.
    let links = read_sky_links_raw(&conn, &path_to_idx, &name_to_idx, &alias_to_path, &mut nodes)?;
    timings.push(("scan_links_and_counts".into(), t3.elapsed().as_millis() as u64));

    Ok(BootSnapshotSky { nodes, links, is_ready, timings_ms: timings })
}

fn read_sky_nodes_raw(conn: &Connection) -> Result<Vec<SkyNodeOut>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, path, library_name, stratum, maturity, origin_type, created_at
             FROM sky_nodes",
        )
        .map_err(|e| format!("prepare nodes: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SkyNodeOut {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                library_name: row.get(3)?,
                // Counts filled in during the links scan pass.
                link_count: 0,
                outgoing_count: 0,
                stratum: row.get::<_, Option<i64>>(4)?,
                maturity: row.get::<_, Option<String>>(5)?,
                origin_type: row.get::<_, Option<String>>(6)?,
                created_at: row.get::<_, Option<i64>>(7)?,
            })
        })
        .map_err(|e| format!("query nodes: {}", e))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("row nodes: {}", e))?);
    }
    Ok(out)
}

fn read_sky_links_raw(
    conn: &Connection,
    path_to_idx: &std::collections::HashMap<String, usize>,
    name_to_idx: &std::collections::HashMap<String, usize>,
    alias_to_path: &std::collections::HashMap<String, String>,
    nodes_mut: &mut [SkyNodeOut],
) -> Result<Vec<SkyLinkOut>, String> {
    let mut stmt = conn
        .prepare("SELECT source_path, target_name, link_type FROM sky_links")
        .map_err(|e| format!("prepare links: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            let source_path: String = row.get(0)?;
            let target_name: String = row.get(1)?;
            let link_type: String = row.get(2)?;
            Ok((source_path, target_name, link_type))
        })
        .map_err(|e| format!("query links: {}", e))?;

    // Reserve roughly the expected capacity to avoid reallocs. For the
    // target universe (232k links) this saves a handful of vec grows.
    let mut out: Vec<SkyLinkOut> = Vec::with_capacity(256 * 1024);

    for r in rows {
        let (source_path, target_name, link_type) = r.map_err(|e| format!("row links: {}", e))?;

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
            "SELECT source_path, source_name, target_name, link_type, library_name,
                    weight, traversal_count, annotation, last_traversed, confidence
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
