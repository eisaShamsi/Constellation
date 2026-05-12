//! Constellation Super Multilingual Search Engine — Phase 1: FTS5 + Structured Queries.
//!
//! Architecture:
//! - SQLite database at {universe}/.constellation/search.db
//! - FTS5 virtual table for lexical search (BM25 ranking)
//! - Metadata table for structured queries (properties, tags, wikilinks)
//! - Arabic Light10 stemming reused from libraries.rs
//! - Incremental indexing via file watcher events
//!
//! Phase 2 will add: ONNX embeddings, semantic search, RRF fusion.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Schema version tracked via `PRAGMA user_version`.
///
/// Increment when the FTS5 tokenizer changes, the `notes_fts` / `notes_vocab`
/// schema changes, or any other setup in `init_db` requires a one-time
/// rebuild of derived data. On boot we drop + recreate the FTS5 chain if
/// the stored version is below this and then issue an `INSERT INTO
/// notes_fts(notes_fts) VALUES('rebuild')` so the new index populates
/// from the existing `note_meta` rows — no filesystem re-walk needed.
///
/// | version | change                                                       |
/// |--------:|--------------------------------------------------------------|
/// |       0 | legacy — notes_fts created with `tokenize='unicode61 ...'`   |
/// |       1 | custom Constellation tokenizer (Arabic Light10 + bigrams)    |
const FTS_SCHEMA_VERSION: i64 = 1;

/// Sky View Write-Time Derivation (MIG-001) schema version.
///
/// Tracks the shape of the `sky_nodes` / `sky_links` tables and their
/// triggers on `note_meta` / `note_links`. Stored in the generic
/// `schema_versions` table rather than `PRAGMA user_version` so it can
/// evolve independently from the FTS version ledger.
///
/// | version | change                                                   |
/// |--------:|----------------------------------------------------------|
/// |       0 | pre-MIG-001 — no sky_* tables; JS buildSkyData() path    |
/// |       1 | sky_nodes + sky_links + triggers + back-fill populator   |
/// |       2 | MIG-002 §1 — note_meta.word_count + created_at columns   |
/// |       3 | MIG-002 §3 — sky_nodes.enrichment_dirty + back-fill of   |
/// |         | word_count / created_at on existing note_meta rows       |
/// |       4 | MIG-002 §4 — SQL-native stratum triggers + one-shot      |
/// |         | back-fill of sky_nodes.stratum for pre-§4 rows           |
/// |       5 | MIG-002 §4 fix (BUG-010) — stratum formula now matches  |
/// |         | inbound on sky_nodes.id (lowercase) instead of .name;    |
/// |         | all stratum values recomputed with correct inbound count |
/// |       6 | MIG-002 §5 — SQL-native maturity triggers + one-shot     |
/// |         | back-fill of sky_nodes.maturity for existing rows        |
/// |       7 | MIG-004 §1 — note_aliases table; alias-aware inbound    |
/// |         | resolution so renames don't drop link counts in stratum |
/// |         | / maturity / map / cache. Schema only.                  |
/// |       8 | MIG-004 §5 — back-fill of frontmatter aliases for       |
/// |         | existing rows. Phase E in sky_backfill::process_batch   |
/// |         | reads each note's `aliases:` YAML list during the same  |
/// |         | file-read pass that produced word_count + created_at.   |
/// |       9 | MIG-004 §6 — STRATUM_SQL_EXPR rewritten to JOIN through |
/// |         | note_aliases on inbound counts. All stratum values      |
/// |         | recomputed under the new formula.                       |
/// |      10 | MIG-004 §7 — MATURITY_SQL_EXPR rewritten through        |
/// |         | note_aliases on all 4 inbound-count gates (canonical /  |
/// |         | wilting / evergreen / sapling). All maturity values     |
/// |         | recomputed under the new formula.                       |
///
/// Bumping the version gates `sky_backfill::maybe_schedule` to repopulate
/// the derived surfaces on next boot. Columns added in v2+ are nullable
/// or defaulted so pre-MIG-002 binaries tolerate the wider schema.
pub(crate) const SKY_SCHEMA_VERSION: i64 = 10;

/// MIG-018 / MIG-019 — `sight_v3_*` cache schema version.
///
/// | Version | Change                                                       |
/// |---------|--------------------------------------------------------------|
/// |       1 | MIG-018 §1A — sight_v3_layout + sight_v3_layout_cursor +     |
/// |         | sight_v3_graph_version tables. Resumable back-fill pattern.  |
/// |       2 | MIG-019 §2A — sight_v3_similarity_edges table for TF-IDF     |
/// |         | content-similarity edges (PJ-035 Milky Way; edge-list        |
/// |         | approach, OOM-prone on large universes — REPLACED in v3).    |
/// |       3 | MIG-019 §2A+§2B redesign — drop sight_v3_similarity_edges;   |
/// |         | add sight_v3_density_grid (single-row BLOB per snapshot).    |
/// |         | Density-field architecture per Concept Paper v1.1 §5.1.      |
/// |         | Memory now bounded by OUTPUT (256² × 4 = 256KB) rather than  |
/// |         | input (millions of candidate pairs). Eisa-directed pivot     |
/// |         | 2026-05-07: "Don't patch it. Solve it."                      |
///
/// Mirrors the SKY_SCHEMA_VERSION pattern: bumping this constant gates
/// the cache-wipe logic in `init_db` below; old rows for any prior
/// version get DELETED on the first boot after the bump, forcing a
/// cold recompute on the next user-driven Sight toggle.
pub(crate) const SIGHT_V3_SCHEMA_VERSION: i64 = 3;

/// MIG-003 Step 1 — `note_meta` schema version. Bumped to 1 when the
/// `cid_cn` column was added and backfilled from frontmatter. Subsequent
/// MIG-003 steps (FK columns on `note_links`/`sky_nodes`/`note_aliases`,
/// PRIMARY-KEY promotion in Step 6) bump this further.
pub(crate) const NOTE_META_SCHEMA_VERSION: i64 = 1;

/// MIG-013 §1C/§1D — `term_vocab.bridge_concept_id` schema version.
///
/// **Status: dead schema as of §1D Option B.** The column was originally
/// designed to hold per-term M11 concept IDs populated at write time
/// (fast path) and via a slow-path `ctse_run_backfill` job. Both paths
/// were retired when CTSE pivoted to query-time concept expansion
/// (Lucene `SynonymGraphFilter` / SQLite FTS5 Method 2 / CLIR
/// query-translation pattern; see `ctse/mod.rs` and `ctse/search.rs`).
/// The column and its v1/v2 migrations are preserved as forward-compat
/// in case a future "deep concept tagging" feature wants the surface.
///
/// | version | change                                                      |
/// |--------:|-------------------------------------------------------------|
/// |       0 | pre-MIG-013 — `term_vocab` exists without bridge column     |
/// |       1 | adds `bridge_concept_id TEXT` column + index. Idempotent    |
/// |         | column-add via `ensure_term_vocab_bridge_column`.           |
/// |       2 | one-shot `UPDATE term_vocab SET bridge_concept_id = '-'    |
/// |         | WHERE bridge_concept_id IS NULL AND term LIKE '%' ||       |
/// |         | CHAR(31) || '%'`. Originally introduced because the         |
/// |         | retired `ctse_run_backfill` job iterated every NULL row    |
/// |         | and would have spent hours on a multi-million-row corpus    |
/// |         | of FTS5 bigrams (joined by `BIGRAM_SEP` = U+001F). After   |
/// |         | the §1D Option B pivot the column is no longer read, so    |
/// |         | this migration is now defensive-only — it ensures any      |
/// |         | future writer that does read the column never sees stale   |
/// |         | NULL bigram rows.                                           |
pub(crate) const TERM_VOCAB_BRIDGE_SCHEMA_VERSION: i64 = 2;

/// MIG-002 §4 — SQL fragment that computes sky_nodes.stratum (1–8) from
/// the same five signals as strata.rs::compute_stratum:
///
///   base = 1 when note_meta.word_count ≤ 50
///          2 when ≤ 200
///          3 otherwise
///   +1   if outgoing active edges ≥ 3
///   +1   if inbound active edges ≥ 5
///   +1   if any outgoing 'generalizes' edge
///   +1   if any outgoing 'causes' or 'supports' edge
///   +1   if distinct inbound sources ≥ 3
///   clamp to [1, 8]
///
/// The expression is correlated on `sky_nodes.path` / `sky_nodes.name`
/// so it works inside any `UPDATE sky_nodes SET stratum = (…)` context.
/// Shared between the §4 triggers in init_db and the one-shot back-fill
/// in sky_backfill.rs — single source of truth, cannot drift.
pub(crate) const STRATUM_SQL_EXPR: &str = "
    MIN(8, MAX(1,
        COALESCE(
            (SELECT CASE
                WHEN word_count <= 50 THEN 1
                WHEN word_count <= 200 THEN 2
                ELSE 3
             END
             FROM note_meta WHERE path = sky_nodes.path),
            1
        )
        + (CASE WHEN (SELECT COUNT(*) FROM note_links
                       WHERE source_path = sky_nodes.path
                         AND status = 'active') >= 3
                THEN 1 ELSE 0 END)
        + (CASE WHEN (SELECT COUNT(*) FROM note_links
                       WHERE status = 'active'
                         AND (target_name = sky_nodes.id
                              OR target_name IN (SELECT alias_lower FROM note_aliases
                                                  WHERE path = sky_nodes.path))) >= 5
                THEN 1 ELSE 0 END)
        + (CASE WHEN EXISTS(SELECT 1 FROM note_links
                             WHERE source_path = sky_nodes.path
                               AND link_type = 'generalizes'
                               AND status = 'active')
                THEN 1 ELSE 0 END)
        + (CASE WHEN EXISTS(SELECT 1 FROM note_links
                             WHERE source_path = sky_nodes.path
                               AND link_type IN ('causes','supports')
                               AND status = 'active')
                THEN 1 ELSE 0 END)
        + (CASE WHEN (SELECT COUNT(DISTINCT source_path) FROM note_links
                       WHERE status = 'active'
                         AND (target_name = sky_nodes.id
                              OR target_name IN (SELECT alias_lower FROM note_aliases
                                                  WHERE path = sky_nodes.path))) >= 3
                THEN 1 ELSE 0 END)
    ))
";

/// MIG-002 §5 — SQL fragment that computes sky_nodes.maturity from the
/// same three signals as maturity.rs::compute_state:
///
///   inbound                  — active links targeting this note
///   days_since_created       — (now - note_meta.created_at) / 86400
///   days_since_modified      — (now - note_meta.modified)    / 86400
///
/// States (order-sensitive — first match wins):
///
///   canonical — inbound ≥ 10 AND days_since_modified ≥ 30
///   wilting   — inbound ≥ 4  AND days_since_created ≥ 7 AND days_since_modified ≥ 90
///   evergreen — inbound ≥ 4  AND days_since_created ≥ 7
///   sapling   — inbound ≥ 1  OR  days_since_created ≥ 2
///   seed      — default
///
/// The expression is correlated on `sky_nodes.path` / `sky_nodes.id` so
/// it works inside any `UPDATE sky_nodes SET maturity = (…)` context.
/// Shared between the §5 triggers in init_db and the back-fill in
/// sky_backfill.rs — single source of truth, cannot drift.
///
/// NOTE on created_at fallback: §89's writer stamps created_at from
/// fs::metadata(..).created() and falls back to `modified` on platforms
/// without a true creation timestamp. On ghost rows (path in DB, file
/// missing) the back-fill leaves created_at NULL; COALESCE to `modified`
/// keeps the arithmetic well-defined (days_since_created becomes 0).
pub(crate) const MATURITY_SQL_EXPR: &str = "
    CASE
        -- canonical: 10+ inbound, untouched 30+ days (authoritative)
        WHEN ((SELECT COUNT(*) FROM note_links
                 WHERE status = 'active'
                   AND (target_name = sky_nodes.id
                        OR target_name IN (SELECT alias_lower FROM note_aliases
                                            WHERE path = sky_nodes.path))) >= 10)
         AND ((strftime('%s','now') -
               COALESCE((SELECT modified FROM note_meta WHERE path = sky_nodes.path), 0))
              / 86400 >= 30)
        THEN 'canonical'

        -- wilting: evergreen-level but untouched 90+ days
        WHEN ((SELECT COUNT(*) FROM note_links
                 WHERE status = 'active'
                   AND (target_name = sky_nodes.id
                        OR target_name IN (SELECT alias_lower FROM note_aliases
                                            WHERE path = sky_nodes.path))) >= 4)
         AND ((strftime('%s','now') -
               COALESCE(
                   (SELECT created_at FROM note_meta WHERE path = sky_nodes.path),
                   (SELECT modified   FROM note_meta WHERE path = sky_nodes.path),
                   strftime('%s','now')))
              / 86400 >= 7)
         AND ((strftime('%s','now') -
               COALESCE((SELECT modified FROM note_meta WHERE path = sky_nodes.path), 0))
              / 86400 >= 90)
        THEN 'wilting'

        -- evergreen: 4+ inbound, created 7+ days ago
        WHEN ((SELECT COUNT(*) FROM note_links
                 WHERE status = 'active'
                   AND (target_name = sky_nodes.id
                        OR target_name IN (SELECT alias_lower FROM note_aliases
                                            WHERE path = sky_nodes.path))) >= 4)
         AND ((strftime('%s','now') -
               COALESCE(
                   (SELECT created_at FROM note_meta WHERE path = sky_nodes.path),
                   (SELECT modified   FROM note_meta WHERE path = sky_nodes.path),
                   strftime('%s','now')))
              / 86400 >= 7)
        THEN 'evergreen'

        -- sapling: 1+ inbound OR created 2+ days ago
        WHEN ((SELECT COUNT(*) FROM note_links
                 WHERE status = 'active'
                   AND (target_name = sky_nodes.id
                        OR target_name IN (SELECT alias_lower FROM note_aliases
                                            WHERE path = sky_nodes.path))) >= 1)
          OR ((strftime('%s','now') -
               COALESCE(
                   (SELECT created_at FROM note_meta WHERE path = sky_nodes.path),
                   (SELECT modified   FROM note_meta WHERE path = sky_nodes.path),
                   strftime('%s','now')))
              / 86400 >= 2)
        THEN 'sapling'

        -- seed: default (fresh + isolated note)
        ELSE 'seed'
    END
";

// ─── Types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub query_embedding: Option<Vec<f32>>,  // pre-computed embedding for semantic search
    pub mode: String,           // "lexical" | "structured" | "semantic" | "hybrid"
    pub filters: Option<SearchFilters>,
    pub limit: Option<u32>,
    pub include_snippet: Option<bool>,
    pub include_headings: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub properties: Option<Vec<PropertyFilter>>,
    pub tags: Option<Vec<String>>,
    pub wikilinks_to: Option<Vec<String>>,
    pub wikilinks_from: Option<Vec<String>>,
    pub mutual: Option<Vec<String>>,
    pub mentions: Option<Vec<String>>,
    pub orphans: Option<bool>,
    pub links_between: Option<Vec<String>>,  // exactly 2 targets
    pub links_all: Option<Vec<String>>,     // incoming + outgoing combined
    pub typed_links: Option<Vec<TypedLinkFilter>>, // cognitive link type queries
    pub library_names: Option<Vec<String>>,
    pub maturity: Option<Vec<String>>,
    pub path_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PropertyFilter {
    pub key: String,
    pub op: String,     // "=" | "!=" | "contains" | "is_empty"
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TypedLinkFilter {
    pub link_type: String,   // supports, contradicts, causes, etc.
    pub target: String,      // target note name
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub library_name: String,
    pub score: f64,
    pub match_type: String,  // "title" | "content" | "property" | "tag" | "wikilink"
    pub snippet: Option<String>,
    pub heading_breadcrumb: Option<Vec<String>>,
    pub modified: u64,
    /// M13 — cross-lingual match badge.
    ///
    /// When the lexical search path (FTS5) surfaces a hit because a
    /// translated lemma matched — not the user's original query — this
    /// carries the bridge term so the UI can render a small "via {lemma}"
    /// pill on the result card. `None` means the hit matched the source
    /// lemma directly (same-language match), or the search path didn't
    /// go through the Lexical Bridge at all (structured / tag / wikilink
    /// / property queries never populate this).
    pub match_via: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchIndexStats {
    pub note_count: u32,
    pub index_size_bytes: u64,
}

// ─── State ─────────────────────────────────────────────────────

pub struct SearchState {
    pub db: Mutex<Option<Connection>>,
}

impl SearchState {
    pub fn new() -> Self {
        SearchState { db: Mutex::new(None) }
    }
}

// ─── Database Setup ────────────────────────────────────────────

pub(crate) fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    Ok(cdir.join("search.db"))
}

/// Append a timestamped line to `<universe>/.constellation/diagnostics.log`.
///
/// Windows Tauri builds are compiled as GUI subsystem so `eprintln!` /
/// `println!` go nowhere even when launched from a terminal. Diagnostics
/// that must be visible after the fact (migration fired? which tokenizer
/// is active? how many `notes_vocab` rows?) therefore need a durable
/// sink the user can open in any editor. Takes the `search.db` path so
/// callers never need to know the Universe root.
///
/// Non-fatal: any failure is swallowed so diagnostics never break the
/// critical path. Also mirrored to `eprintln!` for dev builds where
/// stderr IS attached (e.g. `npm run tauri dev`).
pub(crate) fn diag_log(db_path: &Path, msg: &str) {
    // Still emit to stderr for dev builds and future console-subsystem binaries.
    eprintln!("{}", msg);
    let Some(parent) = db_path.parent() else { return; };
    let log_path = parent.join("diagnostics.log");
    let ts = {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    };
    let line = format!("[{}] {}\n", ts, msg);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        });
}

/// Register the custom Constellation FTS5 tokenizer on a connection.
///
/// Every connection that will run `MATCH` against `notes_fts` — or
/// `CREATE VIRTUAL TABLE ... tokenize='constellation'` — needs this
/// called once. Tokenizer registration is connection-local in SQLite
/// FTS5 (no global registry in the `bundled` build), so callers that
/// open their own connections (e.g. the read-only opens in
/// `libraries::read_index_entries` / `read_term_mentions`) must call
/// this before issuing queries.
///
/// Idempotent within a connection in the sense that repeated calls
/// with the same name register a second time (SQLite shadows the
/// earlier registration); but under normal flow each connection
/// should call this exactly once, right after opening.
pub(crate) fn register_fts5_tokenizer(conn: &mut Connection) -> Result<(), String> {
    let stopwords = Arc::new(crate::libraries::build_stopwords());
    crate::fts5_tokenizer::register_tokenizer::<
        crate::fts5_tokenizer::ConstellationTokenizer,
    >(
        conn,
        crate::fts5_tokenizer::ConstellationGlobal { stopwords },
        "constellation",
    )
}

/// MIG-002: ensure `note_meta` has the `word_count` and `created_at`
/// columns on DBs created by pre-v2 code. Idempotent — checks
/// `PRAGMA table_info` before issuing ALTER TABLE. Safe to run every
/// boot (no writes if columns are present).
fn ensure_note_meta_mig002_columns(conn: &Connection) -> rusqlite::Result<()> {
    let mut have_word_count = false;
    let mut have_created_at = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(note_meta)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for col in rows {
            match col?.as_str() {
                "word_count" => have_word_count = true,
                "created_at" => have_created_at = true,
                _ => {}
            }
        }
    }
    if !have_word_count {
        conn.execute_batch(
            "ALTER TABLE note_meta ADD COLUMN word_count INTEGER NOT NULL DEFAULT 0;",
        )?;
    }
    if !have_created_at {
        conn.execute_batch("ALTER TABLE note_meta ADD COLUMN created_at INTEGER;")?;
    }
    Ok(())
}

/// MIG-003 Step 1 — ensure `note_meta` has the `cid_cn` column.
/// Idempotent: probes `PRAGMA table_info` and ALTERs only if missing.
/// The column starts as `TEXT NOT NULL DEFAULT ''` so existing rows
/// don't violate the constraint. Real `cid_cn` values are populated
/// by `mig003_backfill_cid_cn` (called separately from `init_db`).
/// The UNIQUE index on `cid_cn` is created AFTER the backfill so
/// every row has a real, distinct value when the constraint is
/// applied.
fn ensure_note_meta_mig003_column(conn: &Connection) -> rusqlite::Result<()> {
    let mut have_cid_cn = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(note_meta)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for col in rows {
            if col?.as_str() == "cid_cn" {
                have_cid_cn = true;
            }
        }
    }
    if !have_cid_cn {
        conn.execute_batch(
            "ALTER TABLE note_meta ADD COLUMN cid_cn TEXT NOT NULL DEFAULT '';",
        )?;
    }
    Ok(())
}

/// MIG-013 §1C — ensure `term_vocab` has the `bridge_concept_id` column
/// and its supporting index. Idempotent: probes `PRAGMA table_info` and
/// ALTERs only if the column is missing. The column is nullable
/// (`TEXT` with no NOT NULL) so existing rows accept the wider schema
/// without backfill.
///
/// **Dead schema as of §1D Option B** — nothing reads or writes this
/// column on the live CTSE path. Cross-language search runs entirely at
/// query time via the in-memory `concept_lemmas` map (see
/// `ctse::search::ctse_search_terms_by_concept`). The column survives
/// as forward-compat; the only writer is the v2 sentinel migration
/// below, which one-shot pre-marks bigram rows as defensive cleanup.
fn ensure_term_vocab_bridge_column(conn: &Connection) -> rusqlite::Result<()> {
    let mut have_col = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(term_vocab)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for col in rows {
            if col?.as_str() == "bridge_concept_id" {
                have_col = true;
            }
        }
    }
    if !have_col {
        conn.execute_batch(
            "ALTER TABLE term_vocab ADD COLUMN bridge_concept_id TEXT;",
        )?;
    }
    // Always ensure the index — `CREATE INDEX IF NOT EXISTS` is cheap
    // and covers the case where the column existed but the index was
    // dropped or never created (defensive idempotency).
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_term_vocab_bridge_concept_id
             ON term_vocab (bridge_concept_id);",
    )?;
    Ok(())
}

/// MIG-013 §1D — bulk-sentinel every NULL bigram row.
///
/// On a real Constellation library the FTS5 tokenizer emits both stems
/// AND bigrams (joined by `BIGRAM_SEP` = U+001F) so phrase queries
/// match. Bigrams correctly belong in `term_vocab` (the Index panel
/// and other surfaces consume them), but they are not lexicon-
/// resolvable — the M11 graph keys are single lemmas, never two-word
/// internal sentinel-joined pairs.
///
/// **Why this migration still exists after §1D Option B**: the column
/// is now dead schema, so this UPDATE doesn't affect any live read
/// path. It survives as defensive cleanup for forward-compat. If a
/// future feature ever reactivates `bridge_concept_id` reads, that
/// future feature inherits a column where bigram rows are
/// pre-sentinelled rather than NULL — saving the same iteration cost
/// the original §1D backfill would have hit.
///
/// **Boot-time cost on pre-MIG-013 DBs**: a 7,600-note Arabic+English
/// corpus produces ~50K stems and ~5.7M bigrams. MIG-015 (PJ-001)
/// closed the boot-blocking variant: the migration now runs in a
/// background worker that chunks the UPDATE in 100k-row batches,
/// dropping + re-acquiring the DB mutex around each chunk so other
/// IPC callers see ~10ms availability windows between chunks. Status
/// strip in `MigrationProgressStrip.svelte` shows progress.
///
/// Idempotent: only NULL bigram rows are touched. Real concept ids
/// (`c:…`) and prior `'-'` sentinels are untouched. Re-running the
/// migration is a no-op.
/// MIG-015 §1A — count `term_vocab` rows still requiring the v2 bigram
/// sentinel. Cheap query — exists only to populate the `total` field of
/// the initial progress event so the status-bar strip can show
/// "Migrating term index — 0 / N" before the first chunk runs.
fn count_pending_v2_sentinel_rows(conn: &Connection) -> rusqlite::Result<u64> {
    conn.query_row(
        "SELECT COUNT(*) FROM term_vocab \
         WHERE bridge_concept_id IS NULL \
           AND term LIKE '%' || CHAR(31) || '%'",
        [],
        |row| row.get::<_, i64>(0).map(|n| n as u64),
    )
}

/// MIG-015 §1A — single-chunk variant of `sentinel_bigram_rows`. The
/// caller drives the loop and is responsible for dropping +
/// re-acquiring the DB mutex around each call so concurrent IPC
/// callers see availability windows between chunks. (The original
/// §1A `sentinel_bigram_rows_chunked` helper held the mutex for the
/// whole loop — §1D audit P0 fix.)
///
/// SQLite-portable shape: stock SQLite doesn't support `UPDATE ... LIMIT N`
/// (requires `SQLITE_ENABLE_UPDATE_DELETE_LIMIT`), so we use the
/// idiomatic `UPDATE ... WHERE rowid IN (SELECT rowid ... LIMIT N)`.
///
/// Returns the number of rows affected by this single chunk. 0 means
/// no more pending rows; caller should break out of its loop.
///
/// Crash-recoverable by construction — the WHERE clause excludes
/// already-sentinelled rows, so a re-entry from a prior partial run
/// picks up where it left off without journaling state.
fn sentinel_bigram_rows_chunk(
    conn: &Connection,
    chunk_size: u32,
) -> rusqlite::Result<u64> {
    let affected = conn.execute(
        "UPDATE term_vocab \
            SET bridge_concept_id = '-' \
          WHERE rowid IN ( \
            SELECT rowid FROM term_vocab \
             WHERE bridge_concept_id IS NULL \
               AND term LIKE '%' || CHAR(31) || '%' \
             LIMIT ?1 \
          )",
        rusqlite::params![chunk_size],
    )? as u64;
    Ok(affected)
}

/// MIG-015 §1B — entry point for the deferred v2 sentinel migration.
/// Called from `ensure_search_db_ready` after `init_db` completes and
/// the connection is in state. Mirrors the `sky_backfill::maybe_schedule`
/// pattern: cheap pre-check on the main thread; spawn a worker thread
/// only when work is actually needed.
///
/// Emits `migration:term_vocab_v2` Tauri events with three phases:
///   - `start { total }`         — fired once at the beginning (skipped if total is 0)
///   - `progress { completed, total }` — fired after each non-empty chunk
///   - `done { total }`          — fired once on successful completion
///
/// Stamps `schema_versions.term_vocab_bridge = TERM_VOCAB_BRIDGE_SCHEMA_VERSION`
/// on success. Failure leaves the stamp at its prior value so the next
/// boot retries.
pub fn maybe_schedule_v2_sentinel_migration(app: tauri::AppHandle) {
    use tauri::Manager;

    // Cheap pre-check on the main thread — avoids spawning a worker for
    // the common case (already migrated).
    let state = app.state::<SearchState>();
    let needs_run = {
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        let stored: i64 = conn
            .query_row(
                "SELECT version FROM schema_versions WHERE module = 'term_vocab_bridge'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        stored < TERM_VOCAB_BRIDGE_SCHEMA_VERSION
    };
    if !needs_run {
        return;
    }

    let app_bg = app.clone();
    std::thread::spawn(move || {
        if let Err(e) = run_v2_sentinel_migration(&app_bg) {
            eprintln!("[search] term_vocab v2 sentinel migration task failed: {}", e);
        }
    });
}

/// Body of the deferred migration. Holds the DB mutex for the duration
/// of the chunked loop — each chunk is fast (200-400ms on commodity
/// SSD), so the lock window per chunk is small. Between chunks the
/// runtime briefly yields, allowing other DB callers to interleave.
fn run_v2_sentinel_migration(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri::{Emitter, Manager};

    /// 100k-row chunks. Empirical SSD UPDATE throughput is 250-500k rows/sec,
    /// so each chunk completes in 200-400ms. On a 5.7M-row migration that's
    /// ~50-100 progress updates total — visible motion in the UI without
    /// flooding the Tauri event channel.
    const CHUNK_SIZE: u32 = 100_000;

    let state = app.state::<SearchState>();

    // Re-check inside the worker — the migration could have completed
    // between maybe_schedule's pre-check and this thread starting (e.g.
    // a second ensure_search_db_ready triggered by a window swap).
    let total = {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB not initialized")?;
        let stored: i64 = conn
            .query_row(
                "SELECT version FROM schema_versions WHERE module = 'term_vocab_bridge'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if stored >= TERM_VOCAB_BRIDGE_SCHEMA_VERSION {
            return Ok(());
        }
        count_pending_v2_sentinel_rows(conn).map_err(|e| e.to_string())?
    };

    // No work to do — just stamp and exit. (Fresh DB common case.)
    if total == 0 {
        let mut guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('term_vocab_bridge', ?1, strftime('%s','now'))",
            rusqlite::params![TERM_VOCAB_BRIDGE_SCHEMA_VERSION],
        ).map_err(|e| format!("Failed to stamp schema_versions.term_vocab_bridge: {}", e))?;
        return Ok(());
    }

    // Announce start — frontend strip appears.
    let _ = app.emit("migration:term_vocab_v2", serde_json::json!({
        "phase": "start",
        "total": total,
    }));

    // Run the chunked migration. Drop + re-acquire the DB mutex around
    // each chunk so concurrent IPC callers (note saves, search queries,
    // anything that holds `state.db.lock()`) see ~10ms availability
    // windows between chunks. §1D audit P0 fix: the original §1A
    // helper held the mutex for the whole loop, which contradicted
    // the §1B design contract "between chunks the runtime briefly
    // yields, allowing other DB callers to interleave." The lock is
    // now released between chunks; the brief sleep below ensures the
    // OS scheduler actually gives waiting threads a turn before this
    // worker re-acquires.
    let mut processed: u64 = 0;
    loop {
        let affected = {
            let guard = state.db.lock().map_err(|e| e.to_string())?;
            let conn = guard.as_ref().ok_or("DB not initialized")?;
            sentinel_bigram_rows_chunk(conn, CHUNK_SIZE)
                .map_err(|e| format!("v2 sentinel chunk failed: {}", e))?
        }; // mutex dropped here — other callers can interleave
        if affected == 0 { break; }
        processed += affected;
        let _ = app.emit("migration:term_vocab_v2", serde_json::json!({
            "phase": "progress",
            "completed": processed,
            "total": total,
        }));
        // Brief yield so other DB callers can actually acquire the
        // mutex before this worker re-acquires for the next chunk.
        // 10ms is small relative to the chunk's 200-400ms duration
        // (≤5% slowdown) but long enough to guarantee the OS
        // scheduler interleaves waiting threads.
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Stamp on success. Crash-recoverable by construction — if we
    // failed mid-loop above, we'd have returned an error and the stamp
    // would not land; next boot retries.
    {
        let mut guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('term_vocab_bridge', ?1, strftime('%s','now'))",
            rusqlite::params![TERM_VOCAB_BRIDGE_SCHEMA_VERSION],
        ).map_err(|e| format!("Failed to stamp schema_versions.term_vocab_bridge: {}", e))?;
    }

    // Announce done.
    let _ = app.emit("migration:term_vocab_v2", serde_json::json!({
        "phase": "done",
        "total": processed,
    }));

    Ok(())
}

/// MIG-003 Step 1 — Walk every row in `note_meta`, read the file's
/// frontmatter, ensure `cid_cn:` is present (injecting it via
/// `canonical::ensure_cid_cn` for files that lack it), and populate
/// the `note_meta.cid_cn` column from that value.
///
/// Called at app boot when `schema_versions.note_meta < 1`. Slow on
/// first launch (one disk read per note + a write for every note that
/// lacks `cid_cn:` in frontmatter — could be most of them on a legacy
/// human-named library). Subsequent boots are no-ops because
/// `schema_versions.note_meta` is already at target.
///
/// After this completes successfully, a `CREATE UNIQUE INDEX` on
/// `cid_cn` is added (no duplicates possible because each generated
/// cid_cn embeds a unique timestamp+hex suffix).
///
/// Reports diagnostics via `diag_log` for visibility into long
/// first-launch migrations.
///
/// **Pre-flight cleanup** (handles state from a buggy first run):
///   1. Drop `note_meta` rows pointing at files that no longer exist
///      on disk. Zombie rows from prior deletions/moves.
///   2. Reset any `cid_cn` value that doesn't match the canonical
///      pattern (`YYYYMMDDTHHMMSSZ_KIND_HEX`). Earlier buggy versions
///      of this function stored entire frontmatter blocks into the
///      column; resetting them lets the corrected logic repopulate
///      cleanly.
///
/// **Duplicate handling**: a `cid_cn` collision (two files with the
/// same id, e.g., from File-Explorer copy-paste of a note) blocks the
/// UNIQUE index. On detection, the LATER-modified file gets a fresh
/// cid_cn injected (write + update DB); the earlier-modified file
/// keeps its original cid_cn.
pub(crate) fn mig003_backfill_cid_cn(
    conn: &mut Connection,
    db_dir: &Path,
) -> rusqlite::Result<()> {
    use std::time::Instant;
    let t_start = Instant::now();

    // Canonical-id pattern: YYYYMMDDTHHMMSSZ_KIND_HEX. Used to validate
    // that a cid_cn value looks like an id (vs garbage from the
    // pre-fix bug where entire frontmatter was stored).
    let cid_cn_re = match regex::Regex::new(r"^\d{8}T\d{6}Z_[A-Z0-9]+_[0-9A-F]+$") {
        Ok(r) => r,
        Err(_) => return Ok(()), // unreachable for a valid static pattern
    };

    // ── Snapshot every row's (path, current_cid_cn) BEFORE the heavy
    //    work. Single SELECT outside any transaction. Cheap. ──────
    struct Row {
        path: String,
        current_cid_cn: String,
    }
    let mut rows: Vec<Row> = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT path, cid_cn FROM note_meta")?;
        let mapped = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        for r in mapped {
            let (path, cid_cn) = r?;
            rows.push(Row { path, current_cid_cn: cid_cn });
        }
    }
    diag_log(db_dir, &format!(
        "[search] mig003_backfill_cid_cn: snapshot {} note_meta row(s) in {:?}",
        rows.len(),
        t_start.elapsed(),
    ));

    // ── Phase A: classify rows + read frontmatter for those that need
    //    backfill. NO database writes yet — just gather what we'll do. ─
    enum Plan {
        Skip,                                  // current cid_cn already valid; no DB write
        Delete,                                // file missing on disk
        SetCidCn { new_cid_cn: String },       // populate / repopulate
        Error,                                 // unrecoverable for this row; leave alone
    }
    let mut plans: Vec<(Row, Plan, u64)> = Vec::with_capacity(rows.len());
    let mut pending_for_dedup: Vec<usize> = Vec::new();

    for row in rows {
        let path = Path::new(&row.path);

        // File missing → mark for delete.
        if !path.exists() {
            plans.push((row, Plan::Delete, 0));
            continue;
        }

        let mtime = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Current cid_cn already valid → skip.
        if cid_cn_re.is_match(&row.current_cid_cn) {
            plans.push((row, Plan::Skip, mtime));
            continue;
        }

        // Need to read the file to find the real cid_cn.
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                plans.push((row, Plan::Error, mtime));
                continue;
            }
        };

        let pre_existing = extract_frontmatter_cid_cn(&content);
        let new_cid_cn = match pre_existing {
            Some(c) if cid_cn_re.is_match(&c) => c,
            _ => {
                // Inject via canonical helper. Returns updated content;
                // we extract the new cid_cn from it.
                match crate::canonical::ensure_cid_cn(path, &content) {
                    Ok(updated) => match extract_frontmatter_cid_cn(&updated) {
                        Some(c) if cid_cn_re.is_match(&c) => c,
                        _ => {
                            plans.push((row, Plan::Error, mtime));
                            continue;
                        }
                    },
                    Err(_) => {
                        plans.push((row, Plan::Error, mtime));
                        continue;
                    }
                }
            }
        };
        let idx = plans.len();
        plans.push((row, Plan::SetCidCn { new_cid_cn }, mtime));
        pending_for_dedup.push(idx);
    }

    let phase_a_elapsed = t_start.elapsed();
    diag_log(db_dir, &format!(
        "[search] mig003_backfill_cid_cn: Phase A (classify + read) done in {:?}; {} need backfill",
        phase_a_elapsed,
        pending_for_dedup.len(),
    ));

    // ── Phase B: detect cid_cn collisions across ALL plans (Skip +
    //    SetCidCn). A prior partial run could have populated DB rows
    //    with colliding ids — all show as Skip on the next boot, so
    //    we must scan every populated cid_cn, not just newly-set
    //    ones. The latest-modified file (or lex-larger path on tie)
    //    regenerates and is promoted to SetCidCn so Phase C writes
    //    the new id to the DB. ─────────────────────────────────────
    let mut groups: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, (row, plan, _mtime)) in plans.iter().enumerate() {
        let cid = match plan {
            Plan::Skip => row.current_cid_cn.clone(),
            Plan::SetCidCn { new_cid_cn } => new_cid_cn.clone(),
            _ => continue,
        };
        groups.entry(cid).or_default().push(i);
    }

    let mut duplicates_resolved = 0usize;
    for (_cid, idxs) in groups.into_iter() {
        if idxs.len() < 2 {
            continue;
        }
        // Winner = smallest mtime; tiebreak smaller path. Losers
        // regenerate.
        let mut sorted = idxs.clone();
        sorted.sort_by(|&a, &b| {
            let ma = plans[a].2;
            let mb = plans[b].2;
            ma.cmp(&mb).then_with(|| plans[a].0.path.cmp(&plans[b].0.path))
        });
        let winner = sorted[0];
        for &loser_idx in &sorted[1..] {
            let loser_path = plans[loser_idx].0.path.clone();
            let path_obj = Path::new(&loser_path);
            let content = match std::fs::read_to_string(path_obj) {
                Ok(c) => c,
                Err(_) => {
                    plans[loser_idx].1 = Plan::Error;
                    continue;
                }
            };
            let stripped = strip_cid_cn_line(&content);
            if std::fs::write(path_obj, &stripped).is_err() {
                plans[loser_idx].1 = Plan::Error;
                continue;
            }
            let regenerated = match crate::canonical::ensure_cid_cn(path_obj, &stripped) {
                Ok(updated) => match extract_frontmatter_cid_cn(&updated) {
                    Some(c) if cid_cn_re.is_match(&c) => c,
                    _ => {
                        plans[loser_idx].1 = Plan::Error;
                        continue;
                    }
                },
                Err(_) => {
                    plans[loser_idx].1 = Plan::Error;
                    continue;
                }
            };
            plans[loser_idx].1 = Plan::SetCidCn { new_cid_cn: regenerated };
            duplicates_resolved += 1;
        }
        let _ = winner; // winner stays as-is; no DB write needed
    }

    let phase_b_elapsed = t_start.elapsed();
    diag_log(db_dir, &format!(
        "[search] mig003_backfill_cid_cn: Phase B (dedup) done in {:?}; {} duplicates resolved",
        phase_b_elapsed,
        duplicates_resolved,
    ));

    // ── Phase C: apply all DB changes inside ONE transaction. The
    //    triggers on note_meta cascade per-row, but a single fsync at
    //    COMMIT time is the difference between 30 seconds and 30
    //    minutes on a 7,600-note Universe. ─────────────────────────
    let tx = conn.transaction()?;
    let mut deleted = 0usize;
    let mut updated_skip = 0usize;
    let mut updated_set = 0usize;
    let mut errored = 0usize;
    {
        let mut delete_stmt = tx.prepare("DELETE FROM note_meta WHERE path = ?1")?;
        let mut update_stmt = tx.prepare("UPDATE note_meta SET cid_cn = ?1 WHERE path = ?2")?;
        for (row, plan, _mtime) in &plans {
            match plan {
                Plan::Skip => { updated_skip += 1; }
                Plan::Delete => {
                    if delete_stmt.execute(rusqlite::params![row.path]).is_ok() {
                        deleted += 1;
                    } else {
                        errored += 1;
                    }
                }
                Plan::SetCidCn { new_cid_cn } => {
                    if update_stmt.execute(rusqlite::params![new_cid_cn, row.path]).is_ok() {
                        updated_set += 1;
                    } else {
                        errored += 1;
                    }
                }
                Plan::Error => { errored += 1; }
            }
        }
    }
    tx.commit()?;

    diag_log(db_dir, &format!(
        "[search] mig003_backfill_cid_cn: Phase C (commit) — total={} skipped_already_valid={} backfilled={} deleted_zombies={} duplicates_resolved={} errored={} elapsed={:?}",
        plans.len(),
        updated_skip,
        updated_set,
        deleted,
        duplicates_resolved,
        errored,
        t_start.elapsed(),
    ));

    Ok(())
}

/// MIG-003 Step 1 helper — strip a `cid_cn: ...` line from the YAML
/// frontmatter at the start of file content. Used by the duplicate
/// regeneration path: when two files share a cid_cn, we strip the
/// id from one and let `ensure_cid_cn` re-inject a fresh one. Returns
/// the content unmodified if there's no frontmatter or no cid_cn line.
fn strip_cid_cn_line(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    let leading_offset = content.len() - trimmed.len();
    let after = &trimmed[3..];
    let end = match after.find("\n---") {
        Some(e) => e,
        None => return content.to_string(),
    };
    let fm = &after[..end];
    let body = &after[end..]; // includes the leading `\n---`
    let fm_filtered: String = fm
        .lines()
        .filter(|l| !l.trim().starts_with("cid_cn:"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{}---{}{}",
        &content[..leading_offset],
        fm_filtered,
        body,
    )
}

/// MIG-003 Step 1 — extract `cid_cn:` from a YAML frontmatter block
/// at the start of file content. Returns None if frontmatter is
/// absent / malformed / lacks the field. Mirrors the simple style of
/// `libraries::extract_frontmatter_title`.
fn extract_frontmatter_cid_cn(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return None; }
    let after = &trimmed[3..];
    let end = after.find("\n---")?;
    let fm = &after[..end];
    for line in fm.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("cid_cn:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() { return Some(val.to_string()); }
        }
    }
    None
}

/// MIG-003 Step 1 — Add the UNIQUE index on `note_meta.cid_cn` after
/// the backfill has populated every row. Idempotent — `IF NOT EXISTS`
/// makes re-running safe across boots.
fn ensure_note_meta_mig003_unique_index(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_note_meta_cid_cn ON note_meta(cid_cn);",
    )?;
    Ok(())
}

/// MIG-002: ensure `sky_nodes` has the `enrichment_dirty` column plus
/// its partial index on DBs created by MIG-001 v1 code. Idempotent.
///
/// Default = 1 means every pre-existing sky_nodes row is flagged dirty
/// on ALTER — the §7 enrichment_worker will drain them on first run.
fn ensure_sky_nodes_mig002_columns(conn: &Connection) -> rusqlite::Result<()> {
    let mut have_enrichment_dirty = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(sky_nodes)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for col in rows {
            if col?.as_str() == "enrichment_dirty" {
                have_enrichment_dirty = true;
            }
        }
    }
    if !have_enrichment_dirty {
        conn.execute_batch(
            "ALTER TABLE sky_nodes ADD COLUMN enrichment_dirty INTEGER NOT NULL DEFAULT 1;",
        )?;
    }
    // Always (re)create the partial index — runs after the ALTER on upgrade
    // path, and covers fresh DBs where the CREATE TABLE above already
    // included the column. IF NOT EXISTS makes it idempotent. Partial
    // index on dirty=1 rows only keeps the worker's drain query O(dirty)
    // instead of O(total_nodes); non-dirty rows (steady state) are
    // invisible so the index stays tiny.
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_sky_nodes_enrichment_dirty
             ON sky_nodes(enrichment_dirty) WHERE enrichment_dirty = 1;",
    )?;
    Ok(())
}

// ─── MIG-003 Step 2 — cid_cn columns on dependent tables ────────────────
// Bumped when a fresh re-backfill is required (e.g. trigger fixed).
pub(crate) const DEPENDENT_TABLES_MIG003_VERSION: i64 = 1;

fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in rows {
        if col?.as_str() == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// MIG-003 Step 2 — ensure `note_links` has `source_cid_cn` and
/// `target_cid_cn` columns. Both nullable (target can be unresolved;
/// source could in principle be orphaned). NOT NULL constraint is a
/// Step 6 concern after we're sure backfill leaves no holes.
fn ensure_note_links_mig003_columns(conn: &Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "note_links", "source_cid_cn")? {
        conn.execute_batch("ALTER TABLE note_links ADD COLUMN source_cid_cn TEXT;")?;
    }
    if !column_exists(conn, "note_links", "target_cid_cn")? {
        conn.execute_batch("ALTER TABLE note_links ADD COLUMN target_cid_cn TEXT;")?;
    }
    Ok(())
}

/// MIG-003 Step 2 — ensure `sky_nodes` has the `cid_cn` column.
fn ensure_sky_nodes_mig003_columns(conn: &Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "sky_nodes", "cid_cn")? {
        conn.execute_batch("ALTER TABLE sky_nodes ADD COLUMN cid_cn TEXT;")?;
    }
    Ok(())
}

/// MIG-003 Step 2 — ensure `note_aliases` has the `cid_cn` column.
/// Default '' so existing rows satisfy the NOT NULL constraint; real
/// values come from the backfill.
fn ensure_note_aliases_mig003_columns(conn: &Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "note_aliases", "cid_cn")? {
        conn.execute_batch(
            "ALTER TABLE note_aliases ADD COLUMN cid_cn TEXT NOT NULL DEFAULT '';",
        )?;
    }
    Ok(())
}

/// MIG-003 Step 2 — ensure `note_embeddings` has the `cid_cn` column.
fn ensure_note_embeddings_mig003_columns(conn: &Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "note_embeddings", "cid_cn")? {
        conn.execute_batch("ALTER TABLE note_embeddings ADD COLUMN cid_cn TEXT;")?;
    }
    Ok(())
}

/// MIG-003 Step 3 — boot-time soft re-backfill. Cheap UPDATE that
/// finds 0 rows in steady state (every writer site populates cid_cn
/// directly per Step 3) but repairs any row that escaped — e.g. a
/// dependent-table row written by a path the indexer hadn't reached
/// yet, or a note_meta row whose frontmatter cid_cn was missing at
/// index time. Idempotent and free when there's nothing to do.
pub(crate) fn mig003_step3_soft_rebackfill(
    conn: &mut Connection,
    db_dir: &Path,
) -> rusqlite::Result<()> {
    use std::time::Instant;
    let t = Instant::now();
    let nm = conn.execute(
        "UPDATE note_meta \
         SET cid_cn = COALESCE((SELECT NULLIF(json_extract(properties_json, '$.cid_cn'), '')), cid_cn) \
         WHERE cid_cn IS NULL OR cid_cn = ''",
        [],
    ).unwrap_or(0);
    let nl_src = conn.execute(
        "UPDATE note_links \
         SET source_cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = note_links.source_path) \
         WHERE (source_cid_cn IS NULL OR source_cid_cn = '') \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = note_links.source_path)",
        [],
    ).unwrap_or(0);
    // NOTE: target_cid_cn re-backfill DELIBERATELY OMITTED here. The
    // resolver `LOWER(note_meta.name) = LOWER(target_name)` has no
    // supporting index — running it across 232k+ link rows × 7600+
    // notes was a multi-billion-comparison hang on first boot. New
    // links written after Step 3 ship populate target_cid_cn at INSERT
    // time. Bulk back-fill of pre-existing target_cid_cn=NULL rows
    // is deferred to a later step that builds the necessary index
    // first (or batches with a path-keyed predicate).
    let nl_tgt = 0usize;
    let sn = conn.execute(
        "UPDATE sky_nodes \
         SET cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = sky_nodes.path) \
         WHERE (cid_cn IS NULL OR cid_cn = '') \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = sky_nodes.path)",
        [],
    ).unwrap_or(0);
    let na = conn.execute(
        "UPDATE note_aliases \
         SET cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = note_aliases.path) \
         WHERE cid_cn = '' \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = note_aliases.path)",
        [],
    ).unwrap_or(0);
    let ne = conn.execute(
        "UPDATE note_embeddings \
         SET cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = note_embeddings.path) \
         WHERE (cid_cn IS NULL OR cid_cn = '') \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = note_embeddings.path)",
        [],
    ).unwrap_or(0);
    let total = nm + nl_src + nl_tgt + sn + na + ne;
    if total > 0 {
        diag_log(db_dir, &format!(
            "[search] mig003_step3_soft_rebackfill: repaired note_meta={} note_links src={} tgt={} sky_nodes={} note_aliases={} note_embeddings={} — elapsed={:?}",
            nm, nl_src, nl_tgt, sn, na, ne, t.elapsed(),
        ));
    }
    Ok(())
}

/// MIG-003 Step 2 — single-transaction back-fill of cid_cn on every
/// dependent table by JOINing on the existing `path` columns. Orphan
/// rows (path with no matching note_meta entry) leave cid_cn at its
/// default (NULL or ''). Subsequent steps tighten this.
pub(crate) fn mig003_step2_backfill(
    conn: &mut Connection,
    db_dir: &Path,
) -> rusqlite::Result<()> {
    use std::time::Instant;
    let t = Instant::now();
    let tx = conn.transaction()?;
    let nl_src = tx.execute(
        "UPDATE note_links \
         SET source_cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = note_links.source_path) \
         WHERE (source_cid_cn IS NULL OR source_cid_cn = '') \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = note_links.source_path)",
        [],
    )?;
    let nl_tgt = tx.execute(
        "UPDATE note_links \
         SET target_cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = note_links.target_path) \
         WHERE target_path IS NOT NULL \
           AND (target_cid_cn IS NULL OR target_cid_cn = '') \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = note_links.target_path)",
        [],
    )?;
    let sn = tx.execute(
        "UPDATE sky_nodes \
         SET cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = sky_nodes.path) \
         WHERE (cid_cn IS NULL OR cid_cn = '') \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = sky_nodes.path)",
        [],
    )?;
    let na = tx.execute(
        "UPDATE note_aliases \
         SET cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = note_aliases.path) \
         WHERE cid_cn = '' \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = note_aliases.path)",
        [],
    )?;
    let ne = tx.execute(
        "UPDATE note_embeddings \
         SET cid_cn = (SELECT cid_cn FROM note_meta WHERE note_meta.path = note_embeddings.path) \
         WHERE (cid_cn IS NULL OR cid_cn = '') \
           AND EXISTS (SELECT 1 FROM note_meta WHERE note_meta.path = note_embeddings.path)",
        [],
    )?;
    tx.commit()?;
    diag_log(db_dir, &format!(
        "[search] mig003_step2_backfill: note_links src={} tgt={}, sky_nodes={}, note_aliases={}, note_embeddings={} — elapsed={:?}",
        nl_src, nl_tgt, sn, na, ne, t.elapsed(),
    ));
    Ok(())
}

/// MIG-003 Step 2 — indexes on the new cid_cn columns. UNIQUE where the
/// row-cardinality is one-to-one with note_meta (sky_nodes,
/// note_embeddings). Plain index for note_links (one source has many
/// outgoing edges) and note_aliases (one note has many aliases).
/// SQLite UNIQUE indexes accept multiple NULLs by default, so partial
/// rows (orphan path with no cid_cn) don't block creation.
fn ensure_dependent_tables_mig003_indexes(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_sky_nodes_cid_cn ON sky_nodes(cid_cn);
         CREATE UNIQUE INDEX IF NOT EXISTS idx_note_embeddings_cid_cn ON note_embeddings(cid_cn);
         CREATE INDEX IF NOT EXISTS idx_note_links_source_cid_cn ON note_links(source_cid_cn);
         CREATE INDEX IF NOT EXISTS idx_note_links_target_cid_cn ON note_links(target_cid_cn);
         CREATE INDEX IF NOT EXISTS idx_note_aliases_cid_cn ON note_aliases(cid_cn);",
    )?;
    Ok(())
}

fn init_db(path: &Path) -> Result<Connection, String> {
    let mut conn = Connection::open(path).map_err(|e| format!("Failed to open search.db: {}", e))?;

    // Enable WAL mode for concurrent reads
    conn.execute_batch("PRAGMA journal_mode=WAL;").map_err(|e| e.to_string())?;

    // MIG-002 §6: enable recursive triggers.
    //
    // SQLite defaults `recursive_triggers = OFF` for backwards compat.
    // With it off, when a trigger body writes to another table, that
    // write's triggers are silently skipped. In our case MIG-001's
    // note_meta_sky_ai writes into sky_nodes — and then SQLite refuses
    // to fire the subsequent AFTER INSERT ON note_meta triggers that
    // §4 / §5 added (stratum_ai, maturity_ai). Observed empirically:
    // edit-save on a note leaves stratum + maturity NULL on the new
    // row, while the earlier note_links_sky_stratum_ai / _maturity_ai
    // triggers (which fire from a separate INSERT on note_links that
    // isn't nested inside another trigger) work fine.
    //
    // Turning this ON makes chained trigger semantics match the
    // intuitive model: every AFTER INSERT trigger on note_meta fires
    // for every note_meta INSERT, regardless of whether an earlier
    // trigger in the chain already wrote to some other table.
    conn.execute_batch("PRAGMA recursive_triggers=ON;")
        .map_err(|e| format!("recursive_triggers pragma: {}", e))?;

    // ─── Register the custom FTS5 tokenizer ──────────────────────────
    // Must happen BEFORE any `CREATE VIRTUAL TABLE ... tokenize='constellation'`
    // so SQLite can resolve the tokenizer name. Safe to call on a
    // connection that has never seen FTS5 — it only wires up an
    // in-memory pointer on the connection; no DB state changes.
    register_fts5_tokenizer(&mut conn)?;

    // ─── FTS schema migration ────────────────────────────────────────
    // Old databases have `notes_fts` created with
    //   tokenize='unicode61 remove_diacritics 2'
    // `CREATE VIRTUAL TABLE IF NOT EXISTS` below would NOT change an
    // existing table's tokenizer — it silently skips. So if the stored
    // `PRAGMA user_version` is below the current FTS schema version we
    // drop the FTS5 chain, let the CREATE statements below rebuild it
    // with the new tokenizer, and then issue a `rebuild` command to
    // repopulate it from `note_meta` (no filesystem walk needed — FTS5
    // re-indexes from the content table). See FTS_SCHEMA_VERSION above
    // for the version ledger.
    let stored_version: i64 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| format!("PRAGMA user_version failed: {}", e))?;
    let needs_fts_rebuild = stored_version < FTS_SCHEMA_VERSION;
    diag_log(path, &format!(
        "[search] init_db: PRAGMA user_version={} (target {}) — rebuild {}",
        stored_version,
        FTS_SCHEMA_VERSION,
        if needs_fts_rebuild { "NEEDED (dropping notes_fts/notes_vocab)" } else { "skipped (already current)" },
    ));

    // Generic module version ledger — lives alongside the FTS
    // `PRAGMA user_version` slot so additional WTD surfaces (MIG-001
    // Sky View, future Sight/Map/counts) can evolve their schema
    // independently of the FTS tokenizer version. Rows are tiny; this
    // table is pure metadata.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS schema_versions (
            module TEXT PRIMARY KEY,
            version INTEGER NOT NULL,
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
    ").map_err(|e| format!("Failed to create schema_versions: {}", e))?;
    let stored_sky_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'sky'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let needs_sky_rebuild = stored_sky_version < SKY_SCHEMA_VERSION;
    diag_log(path, &format!(
        "[search] init_db: schema_versions.sky={} (target {}) — sky {}",
        stored_sky_version,
        SKY_SCHEMA_VERSION,
        if needs_sky_rebuild { "REBUILD NEEDED (MIG-001 Step 2+)" } else { "current" },
    ));
    if needs_fts_rebuild {
        // Drop notes_vocab first (it depends on notes_fts). IF EXISTS so
        // this is a no-op on fresh DBs.
        conn.execute_batch("
            DROP TABLE IF EXISTS notes_vocab;
            DROP TABLE IF EXISTS notes_fts;
        ").map_err(|e| format!("Failed to drop old FTS chain during migration: {}", e))?;
    }

    // Create metadata table.
    //
    // MIG-002 (v2): `word_count` and `created_at` columns are denormalized
    // signals read by the SQL-native stratum + maturity triggers on
    // sky_nodes. `word_count` is stamped by the Rust writer on every note
    // save (see index_note); `created_at` is stamped on INSERT from
    // fs::metadata, falling back to `modified` when the platform lacks a
    // true creation timestamp (ReFS, FAT32). Back-fill populates both on
    // existing rows via sky_backfill.rs.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_meta (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            library_name TEXT NOT NULL,
            modified INTEGER NOT NULL,
            content_hash TEXT,
            properties_json TEXT DEFAULT '{}',
            tags_json TEXT DEFAULT '[]',
            outgoing_links_json TEXT DEFAULT '[]',
            headings_json TEXT DEFAULT '[]',
            body_text TEXT DEFAULT '',
            word_count INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER
        );
    ").map_err(|e| format!("Failed to create note_meta: {}", e))?;

    // MIG-002: idempotent ALTER for pre-v2 DBs. SQLite lacks IF NOT EXISTS
    // on ADD COLUMN, so we probe table_info. Cheap (one row per column,
    // runs once per boot).
    ensure_note_meta_mig002_columns(&conn)
        .map_err(|e| format!("Failed to ensure note_meta MIG-002 columns: {}", e))?;

    // MIG-003 Step 1 — `cid_cn` column on `note_meta`. Idempotent
    // schema add; the actual cid_cn values are populated by the
    // backfill below (one-shot, gated by schema_versions.note_meta).
    ensure_note_meta_mig003_column(&conn)
        .map_err(|e| format!("Failed to ensure note_meta MIG-003 column: {}", e))?;
    let stored_note_meta_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'note_meta'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stored_note_meta_version < NOTE_META_SCHEMA_VERSION {
        diag_log(path, &format!(
            "[search] init_db: schema_versions.note_meta={} (target {}) — MIG-003 backfill needed",
            stored_note_meta_version,
            NOTE_META_SCHEMA_VERSION,
        ));
        mig003_backfill_cid_cn(&mut conn, path)
            .map_err(|e| format!("Failed to backfill cid_cn for MIG-003: {}", e))?;
        // Add UNIQUE index now that every row has a real cid_cn.
        ensure_note_meta_mig003_unique_index(&conn)
            .map_err(|e| format!("Failed to add UNIQUE index on cid_cn: {}", e))?;
        // Stamp completion. Future boots short-circuit the backfill.
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('note_meta', ?1, strftime('%s','now'))",
            rusqlite::params![NOTE_META_SCHEMA_VERSION],
        ).map_err(|e| format!("Failed to stamp schema_versions.note_meta: {}", e))?;
        diag_log(path, &format!(
            "[search] init_db: schema_versions.note_meta stamped to {} after MIG-003 Step 1 backfill",
            NOTE_META_SCHEMA_VERSION,
        ));
    } else {
        diag_log(path, &format!(
            "[search] init_db: schema_versions.note_meta={} (target {}) — MIG-003 backfill skipped (already done)",
            stored_note_meta_version,
            NOTE_META_SCHEMA_VERSION,
        ));
    }

    // MIG-021 §1A — Sources subsystem schema. Idempotent ALTER for the
    // `sources` column on `note_meta` + `CREATE TABLE IF NOT EXISTS`
    // for the `sources_suggestions` queue. Both no-op on a DB that
    // already has them. See `crate::sources` for the full subsystem.
    crate::sources::ensure_note_meta_sources_column(&conn)
        .map_err(|e| format!("Failed to ensure note_meta.sources column (MIG-021): {}", e))?;
    crate::sources::ensure_sources_suggestions_table(&conn)
        .map_err(|e| format!("Failed to create sources_suggestions table (MIG-021): {}", e))?;

    // MIG-021v2 §1A' — Content-type subsystem schema. Idempotent
    // ALTER for the new `content_type` column on `note_meta`. No-op
    // on a DB that already has it. The classifier's vertical-axis
    // suggestions land in the same `sources_suggestions` queue (axis
    // tag distinguishes), so no second queue table is needed.
    crate::sources::ensure_note_meta_content_type_column(&conn)
        .map_err(|e| format!("Failed to ensure note_meta.content_type column (MIG-021v2): {}", e))?;

    // MIG-022 §B.1 — Note state history (temporal axis). Persists
    // changes to epistemic frontmatter fields per the gap-analysis
    // §6.3 recommendation. Idempotent table + index creation; runs
    // after the sources/content_type columns exist so the trigger
    // (added in §B.2) can reference them. The foreign key
    // `note_state_history.note_path → note_meta(path)` cascades on
    // delete.
    crate::cece::history::ensure_note_state_history_table(&conn)
        .map_err(|e| format!("Failed to create note_state_history table (MIG-022 §B.1): {}", e))?;

    // MIG-022 §B.2 — Trigger fires AFTER UPDATE on note_meta when
    // any of the watched epistemic-field columns (sources,
    // content_type, properties_json) actually changed (WHEN guard
    // skips no-op writes — the canonical SQLite footgun the
    // cross-check warned about). Captures the old + new values for
    // each changed field as a single JSON-diff row in
    // note_state_history. Idempotent via CREATE TRIGGER IF NOT EXISTS.
    crate::cece::history::ensure_note_state_history_trigger(&conn)
        .map_err(|e| format!("Failed to create note_state_history trigger (MIG-022 §B.2): {}", e))?;

    // Create embeddings table for semantic search (Phase 2)
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_embeddings (
            path TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            dimensions INTEGER NOT NULL DEFAULT 384,
            model_id TEXT DEFAULT 'all-MiniLM-L6-v2'
        );
    ").map_err(|e| format!("Failed to create note_embeddings: {}", e))?;

    // (MIG-012 `term_embeddings` table retired by MIG-013 §1C: the
    // per-library term-embedding pipeline was replaced by the CTSE
    // Bridge Adapter — `term_vocab.bridge_concept_id` resolves to one
    // of M11's ~20K curated concepts whose vectors are baked at build
    // time. The retired table is left untouched on disk for any DB
    // that still has it; future cleanup may DROP it but is not
    // required for correctness.)

    // MIG-012-fix-8 — shadow vocabulary table for fast term enumeration.
    // Replaces direct queries against `notes_vocab` (fts5vocab virtual
    // table) which walks every doc-list per query and times out on
    // large libraries (Boss reported 20+ minutes on a 7,600-note
    // library; pathological at 10K+ notes).
    //
    // CLAUDE.md Rule 8 (Write-Time Derivation): vocabulary is
    // materialized into a regular indexed table on every save, then
    // queried at read time. The maintenance happens in MIG-013 §1C's
    // `ctse::hooks::on_note_indexed`, called from `reindex_single_note`
    // post-COMMIT. `term_vocab.term` matches the FTS5 token namespace
    // because both use `fts5_tokenizer::tokenize_to_vec` over the same
    // body_text.
    //
    // (MIG-013 §1A-§1C retired the prior bulk `populate_term_vocab`
    // bootstrap. Existing libraries start with a sparse term_vocab and
    // grow it incrementally as notes save. A first-fill walk over
    // `note_meta.body_text` is queued for §1D so the §1D Boss-test
    // doesn't depend on per-note edits.)
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS term_vocab (
            term TEXT PRIMARY KEY,
            doc_count INTEGER NOT NULL,
            total_count INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_term_vocab_total_count
            ON term_vocab (total_count DESC);
    ").map_err(|e| format!("Failed to create term_vocab: {}", e))?;

    // MIG-013 §1C/§1D — `term_vocab.bridge_concept_id` column + index +
    // bigram-sentinel migration. Each step is gated independently
    // against `schema_versions.term_vocab_bridge` so we re-enter cleanly
    // on partial-migration recovery (e.g. crash between v1 and v2).
    let stored_term_vocab_bridge_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'term_vocab_bridge'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Step v0→v1 — column + index. Idempotent on every boot
    // regardless of stored version (the helper probes `PRAGMA
    // table_info` and only ALTERs when missing). Cheap.
    ensure_term_vocab_bridge_column(&conn)
        .map_err(|e| format!("Failed to ensure term_vocab.bridge_concept_id: {}", e))?;

    // Step v1→v2 — bulk-sentinel bigram rows. MIG-015 (PJ-001) defers
    // this step off the boot critical path. On Boss-equivalent libraries
    // (~5.7M term_vocab rows), the bulk UPDATE blocks for 30-90 sec
    // with no UI feedback. We now only DETECT pending here; the actual
    // UPDATE runs in a deferred async task scheduled from
    // `ensure_search_db_ready` via `maybe_schedule_v2_sentinel_migration`.
    //
    // Schema-version stamping also moves into the deferred task's
    // success path. If the user kills the app mid-migration,
    // schema_versions.term_vocab_bridge stays at its prior value and
    // the next boot picks up where left off via the WHERE clause's
    // `bridge_concept_id IS NULL` filter (crash-recoverable by
    // construction — no journal table, no checkpointing).
    if stored_term_vocab_bridge_version < TERM_VOCAB_BRIDGE_SCHEMA_VERSION {
        diag_log(path, &format!(
            "[search] init_db: term_vocab v2 sentinel migration deferred to async task (stored version = {})",
            stored_term_vocab_bridge_version,
        ));
    }

    // MIG-012 — Search history. Per-Universe (this database is per-
    // Universe). Boss-approved Q3.B. Capped at 200 rows by application
    // logic on each write (FIFO eviction).
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS index_search_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            query TEXT NOT NULL UNIQUE,
            last_used INTEGER NOT NULL,
            use_count INTEGER NOT NULL DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_index_search_history_last_used
            ON index_search_history (last_used DESC);
    ").map_err(|e| format!("Failed to create index_search_history: {}", e))?;

    // Create FTS5 virtual table for full-text search.
    //
    // Uses the custom 'constellation' tokenizer (registered above) so
    // the stored tokens are already stemmed forms:
    //   * Arabic Light10 collapses the ~452k surface forms observed on
    //     a 7,600-note Arabic-heavy Universe to ~30-60k stems.
    //   * Multi-language stemmers (Persian / Hebrew / Cyrillic /
    //     Devanagari / German / Spanish / Portuguese / French / Turkish /
    //     English) each collapse their own inflections.
    //   * Bigrams are emitted as colocated tokens, joined by the
    //     `fts5_tokenizer::BIGRAM_SEP` sentinel byte.
    //   * `MATCH` queries are stemmed through the same tokenizer, so
    //     `MATCH 'الكتاب'` and `MATCH 'كتب'` both land on the stem 'كتب'.
    conn.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            name,
            body_text,
            content=note_meta,
            content_rowid=rowid,
            tokenize='constellation'
        );
    ").map_err(|e| format!("Failed to create notes_fts: {}", e))?;

    // MIG-003 Step 1 hardening: drop the pre-MIG-003 `note_meta_au`
    // trigger so the WHEN-clause-gated version below replaces it on
    // existing DBs. Without this, an existing user database keeps the
    // old trigger with no WHEN clause, defeating the gate. Pure no-op
    // on a fresh DB.
    conn.execute_batch("DROP TRIGGER IF EXISTS note_meta_au;")
        .map_err(|e| format!("Failed to drop pre-MIG-003 note_meta_au trigger: {}", e))?;

    // Triggers to keep FTS in sync with note_meta. The AU trigger
    // is gated on `name` or `body_text` actually changing — without
    // the gate, a column-add migration like MIG-003 (which only
    // changes `cid_cn`) would force full FTS5 retokenization for
    // every note in the library, hanging Phase C indefinitely on
    // a 7,000-note Universe.
    conn.execute_batch("
        CREATE TRIGGER IF NOT EXISTS note_meta_ai AFTER INSERT ON note_meta BEGIN
            INSERT INTO notes_fts(rowid, name, body_text) VALUES (new.rowid, new.name, new.body_text);
        END;
        CREATE TRIGGER IF NOT EXISTS note_meta_ad AFTER DELETE ON note_meta BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, name, body_text) VALUES('delete', old.rowid, old.name, old.body_text);
        END;
        CREATE TRIGGER IF NOT EXISTS note_meta_au
        AFTER UPDATE ON note_meta
        WHEN OLD.name IS NOT NEW.name
          OR OLD.body_text IS NOT NEW.body_text
        BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, name, body_text) VALUES('delete', old.rowid, old.name, old.body_text);
            INSERT INTO notes_fts(rowid, name, body_text) VALUES (new.rowid, new.name, new.body_text);
        END;
    ").map_err(|e| format!("Failed to create FTS triggers: {}", e))?;

    // ─── Index Panel vocabulary view ─────────────────────────────────────
    // fts5vocab exposes the sorted term dictionary that FTS5 already
    // maintains on-disk as triggers update `notes_fts`. Row mode:
    //   (term TEXT, doc INTEGER, cnt INTEGER)
    //   * doc — number of distinct notes containing the term
    //   * cnt — total occurrences across all notes
    // This replaces the custom index_terms/index_mentions/index_meta tables:
    // the Index panel reads directly from the already-maintained FTS5 index,
    // no separate tokenization or aggregation pass is needed, and edits to
    // notes update the vocab transparently via the existing FTS5 triggers.
    conn.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_vocab USING fts5vocab(notes_fts, 'row');
    ").map_err(|e| format!("Failed to create notes_vocab: {}", e))?;

    // Indexes for structured queries
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_note_library ON note_meta(library_name);
        CREATE INDEX IF NOT EXISTS idx_note_modified ON note_meta(modified);
        CREATE INDEX IF NOT EXISTS idx_note_name ON note_meta(name);
    ").map_err(|e| format!("Failed to create indexes: {}", e))?;

    // Covering index for the boot-path projection:
    //   SELECT name, path, library_name FROM note_meta
    // Without this, SQLite does a full table scan and reads the wide
    // rows (body_text + *_json blobs, ~80 MB on a 7,600-note Universe)
    // just to project three narrow TEXT columns. With the covering
    // index, the planner does an index-only scan over ~200 KB of index
    // pages. Measured 2026-04-16: brings `read_notes` from 8021 ms to
    // low-millis on cold boot. See lab/boot-perf/boot-bundle-cold-start.md.
    //
    // `IF NOT EXISTS` + no version bump means this index is picked up
    // on the next app launch without deleting or rebuilding the user's
    // existing search.db.
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_note_boot_snapshot
            ON note_meta(name, path, library_name);
    ").map_err(|e| format!("Failed to create idx_note_boot_snapshot: {}", e))?;

    // ─── Living Link System (Knowledge Formulation) ─────────────────────
    // note_links: stores typed, directed, annotated links with lifecycle data.
    // Source of truth: LINK files on disk. This table is the fast index.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_path TEXT NOT NULL,
            source_name TEXT NOT NULL,
            target_path TEXT,
            target_name TEXT NOT NULL,
            link_type TEXT NOT NULL DEFAULT 'relates',
            annotation TEXT DEFAULT '',
            confidence TEXT DEFAULT 'hypothesis',
            weight REAL DEFAULT 1.0,
            created TEXT DEFAULT '',
            last_traversed TEXT DEFAULT '',
            traversal_count INTEGER DEFAULT 0,
            library_name TEXT DEFAULT '',
            status TEXT DEFAULT 'active',
            UNIQUE(source_path, target_name, link_type)
        );
        CREATE INDEX IF NOT EXISTS idx_link_source ON note_links(source_path);
        CREATE INDEX IF NOT EXISTS idx_link_target ON note_links(target_name);
        CREATE INDEX IF NOT EXISTS idx_link_type ON note_links(link_type);
        CREATE INDEX IF NOT EXISTS idx_link_weight ON note_links(weight);
        CREATE INDEX IF NOT EXISTS idx_link_confidence ON note_links(confidence);
        CREATE INDEX IF NOT EXISTS idx_link_status ON note_links(status);
        CREATE INDEX IF NOT EXISTS idx_link_last_traversed ON note_links(last_traversed);
        CREATE INDEX IF NOT EXISTS idx_link_traversal_count ON note_links(traversal_count);
    ").map_err(|e| format!("Failed to create note_links: {}", e))?;

    // Drop any leftover tables from the aborted custom-index experiment
    // (2026-04-16). The Index panel now reads directly from the FTS5 vocab
    // virtual table `notes_vocab` above; these tables are no longer used.
    conn.execute_batch("
        DROP TABLE IF EXISTS index_mentions;
        DROP TABLE IF EXISTS index_terms;
        DROP TABLE IF EXISTS index_meta;
    ").map_err(|e| format!("Failed to drop obsolete index tables: {}", e))?;

    // ─── Sky View Write-Time Derivation (MIG-001 Step 2) ────────────────
    // sky_nodes + sky_links are the persisted derived surface for the Sky
    // View graph. Step 2 ships the schema only — the triggers that keep
    // them synced with note_meta / note_links land in Steps 3–4, and the
    // back-fill populator in Step 5. Until then these tables stay empty
    // and the JS buildSkyData() path still drives the UI.
    //
    // Design notes:
    // - sky_nodes keyed by `path` (stable across renames of the display
    //   name) matches note_meta's PK. `id` is the lower-cased name that
    //   the frontend uses as the join key to sky_links (Invariant 3 from
    //   the Phase-1 doc — cross-library name collision accepted as-is).
    // - sky_links.target_name is name-based (not path-based) because
    //   wikilinks target names and the resolver lives elsewhere. Matches
    //   the current SkyLink.target shape.
    // - UNIQUE(source_path, target_name, link_type) is the dedup
    //   invariant — duplicate wikilinks in a note collapse to one edge
    //   (Invariant 2). `count` tracks the pre-dedup multiplicity so we
    //   don't lose information.
    // - No FOREIGN KEYs: back-fill order can insert links before nodes
    //   during a chunked rebuild, and SQLite FK enforcement is off by
    //   default. The triggers on note_meta / note_links maintain
    //   integrity via the UNIQUE constraints plus ON DELETE cascades in
    //   the trigger bodies (Steps 3–4).
    // MIG-001 Step 7 audit finding: stratum/maturity/origin_type are
    // compute-on-demand in the current architecture — strata.rs,
    // maturity.rs, and provenance.rs are pure filesystem scanners that
    // return results to the frontend without persisting them anywhere.
    // There's no existing write path that could feed a WTD trigger.
    //
    // Decision: keep the columns as forward-compat placeholders (so the
    // shape of sky_nodes matches the SkyNode TypeScript type and a
    // future MIG-002 can flip them to WTD without a schema change), but
    // leave them NULL for MIG-001 v1. Frontend continues calling
    // compute_note_strata / compute_note_maturity separately after the
    // new cache_boot_snapshot_sky IPC (Step 8) returns the base graph.
    // Scope discipline per LL-023: enrichment migration is orthogonal
    // to Sky View's primary perf pain (node+edge serialization) and
    // belongs in its own MIG-002.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS sky_nodes (
            path TEXT PRIMARY KEY,
            id TEXT NOT NULL,
            name TEXT NOT NULL,
            library_name TEXT NOT NULL,
            link_count INTEGER NOT NULL DEFAULT 0,
            outgoing_count INTEGER NOT NULL DEFAULT 0,
            -- Enrichment columns — populated in MIG-002.
            -- stratum / maturity are computed SQL-natively by triggers
            -- installed in §4 / §5. origin_type requires a recursive
            -- derives-from chain walk and is populated by
            -- enrichment_worker.rs (§7) off the enrichment_dirty flag.
            stratum TEXT,
            maturity TEXT,
            origin_type TEXT,
            -- enrichment_dirty: 1 = origin_type needs recomputation by
            -- the background worker. Set to 1 by (a) row insert (fresh
            -- row = origin unknown), (b) note_links triggers on any
            -- derives-from edge change affecting this row (§8),
            -- (c) rename of a note participating in any derives-from
            -- edge (§6 rename AU). Cleared by the worker after write.
            enrichment_dirty INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER,
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_sky_nodes_library ON sky_nodes(library_name);
        CREATE INDEX IF NOT EXISTS idx_sky_nodes_id ON sky_nodes(id);
        -- NOTE: the idx_sky_nodes_enrichment_dirty partial index is created
        -- inside ensure_sky_nodes_mig002_columns below, not here. On DBs
        -- upgraded from MIG-001 v1 the column does not exist yet when
        -- this batch runs (CREATE TABLE IF NOT EXISTS is a no-op on
        -- existing tables), and referencing it in a CREATE INDEX would
        -- abort the whole execute_batch with a missing-column error.

        CREATE TABLE IF NOT EXISTS sky_links (
            source_path TEXT NOT NULL,
            target_name TEXT NOT NULL,
            link_type TEXT NOT NULL DEFAULT '',
            weight REAL NOT NULL DEFAULT 0,
            count INTEGER NOT NULL DEFAULT 1,
            UNIQUE(source_path, target_name, link_type)
        );
        CREATE INDEX IF NOT EXISTS idx_sky_links_source ON sky_links(source_path);
        CREATE INDEX IF NOT EXISTS idx_sky_links_target ON sky_links(target_name);
        -- Deliberately no index on link_type alone: ~7 distinct values across
        -- 217k rows makes it non-selective, and all current queries filter by
        -- source_path or target_name (covered above) with link_type as payload.
        -- Reinstate if a pure `WHERE link_type=?` query ever shows up.
    ").map_err(|e| format!("Failed to create sky_* tables: {}", e))?;

    // MIG-002: idempotent ALTER for pre-v2 DBs that already have
    // sky_nodes from MIG-001. Adds enrichment_dirty column + partial
    // index. No-op on fresh DBs (column already in CREATE TABLE).
    ensure_sky_nodes_mig002_columns(&conn)
        .map_err(|e| format!("Failed to ensure sky_nodes MIG-002 columns: {}", e))?;

    // ─── MIG-004 §1: note_aliases table ─────────────────────────────────
    //
    // Persists every alias under which a note can be addressed by an
    // inbound wikilink. Three sources feed the table:
    //
    //   'frontmatter' — the note's own `aliases:` YAML list, repopulated
    //                   by index_note on every save (DELETE-by-source +
    //                   INSERT, partition isolates from other sources)
    //   'rename'      — the prior display name of a note, stamped here
    //                   when the user renames so old wikilinks still
    //                   resolve (the central fix this migration delivers)
    //   'import'      — Obsidian-imported aliases, distinct provenance
    //                   for any future audit / migration tools
    //
    // alias_lower is Arabic-normalized + lowercased at insert time —
    // matches `extract_wikilinks` normalization so the inbound JOIN
    // against `note_links.target_name` is a direct equality with no
    // per-row work.
    //
    // Composite PK = idempotent (path, alias_lower) inserts via
    // INSERT OR IGNORE. idx_note_aliases_lookup is the hot path —
    // every alias-aware inbound subquery in MIG-004 §6/§7 hits it.
    //
    // NOTE on `source`: informational only. Composite PK + INSERT OR
    // IGNORE means the FIRST writer wins permanently — if §3 stamps
    // ('rename') and the user then adds the same alias to frontmatter
    // (§2 / §5 source='frontmatter'), the row keeps source='rename'.
    // Resolution semantics in §6/§7/§8/§9 do NOT filter by source —
    // they match `alias_lower` regardless of source. Future code that
    // expects "frontmatter rows mirror current YAML" should consult
    // the file's frontmatter directly, not this column.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_aliases (
            path        TEXT NOT NULL,
            alias_lower TEXT NOT NULL,
            added_at    INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            source      TEXT NOT NULL,
            PRIMARY KEY (path, alias_lower)
        );
        CREATE INDEX IF NOT EXISTS idx_note_aliases_lookup
            ON note_aliases(alias_lower);
        CREATE INDEX IF NOT EXISTS idx_note_aliases_path
            ON note_aliases(path);
    ").map_err(|e| format!("Failed to create note_aliases: {}", e))?;

    // ─── Sky-link triggers (MIG-001 Step 3) ─────────────────────────────
    // Keep sky_links in lock-step with note_links. Triggers fire on every
    // write to note_links regardless of which of the 9 writer sites
    // (search.rs 623/1389/1743/1812/1833/1841/1869/1891/2033) did it —
    // that's the whole point of using the DB as the integration boundary
    // instead of threading hooks through every Rust call site.
    //
    // Invariant 4 (archived links excluded): the AI / AU triggers only
    // insert when NEW.status = 'active'. Archiving a link fires AU with
    // the old row still reachable via OLD; the DELETE clause removes it.
    //
    // Invariant 2 (dedup by source→target:type): note_links already has
    // UNIQUE(source_path, target_name, link_type), so the dedup happens
    // upstream. sky_links inherits the same UNIQUE constraint so an
    // accidental duplicate trigger fire is idempotent.
    //
    // Update strategy: AU deletes the OLD row by its key and re-inserts
    // NEW if active. This handles (a) status transitions active↔archived,
    // (b) weight changes, (c) rare edits where the key itself changes.
    // A WHEN guard skips the body entirely for metadata-only updates —
    // traversal bumps (last_traversed, traversal_count) and confidence
    // edits don't affect the sky_links shape, so firing the trigger body
    // on every SV click would be pure write amplification.
    //
    // Weight default mirrors note_links.weight DEFAULT 1.0 so a freshly
    // created link with a NULL weight lands in sky_links as 1.0 (the
    // Living Link "birth weight"), not 0 (which on the weight scale
    // means "dormant/dead").
    //
    // INSERT (not INSERT OR REPLACE) is correct here: AI only fires on
    // genuinely new rows, and AU explicitly DELETEs the OLD key before
    // inserting NEW — so the UNIQUE-constraint resolver would never
    // trigger and OR REPLACE would only add overhead.
    // INSERT OR IGNORE (not plain INSERT) on AI / AU re-insert: defends
    // against races with the back-fill populator (sky_backfill.rs) and
    // the AU→delete-then-insert pattern when another writer briefly
    // repopulates the same key. SQLite serializes writes, so the window
    // is narrow, but the UNIQUE constraint would still raise on the
    // rare overlap. OR IGNORE is idempotent: benign no-op when the row
    // already exists, same observable state either way.
    //
    // Weight-divergence check: sky_backfill.rs uses the same
    // `COALESCE(weight, 1.0)` convention as these triggers, so OR IGNORE
    // silently keeping the back-fill's weight never produces a different
    // value from what the trigger would have written.
    conn.execute_batch("
        CREATE TRIGGER IF NOT EXISTS note_links_sky_ai
        AFTER INSERT ON note_links
        WHEN NEW.status = 'active'
        BEGIN
            INSERT OR IGNORE INTO sky_links (source_path, target_name, link_type, weight)
            VALUES (NEW.source_path, NEW.target_name, NEW.link_type, COALESCE(NEW.weight, 1.0));
        END;

        CREATE TRIGGER IF NOT EXISTS note_links_sky_ad
        AFTER DELETE ON note_links
        BEGIN
            DELETE FROM sky_links
            WHERE source_path = OLD.source_path
              AND target_name = OLD.target_name
              AND link_type = OLD.link_type;
        END;

        CREATE TRIGGER IF NOT EXISTS note_links_sky_au
        AFTER UPDATE ON note_links
        WHEN OLD.source_path IS NOT NEW.source_path
          OR OLD.target_name IS NOT NEW.target_name
          OR OLD.link_type   IS NOT NEW.link_type
          OR OLD.status      IS NOT NEW.status
          OR COALESCE(OLD.weight, 1.0) IS NOT COALESCE(NEW.weight, 1.0)
        BEGIN
            DELETE FROM sky_links
            WHERE source_path = OLD.source_path
              AND target_name = OLD.target_name
              AND link_type = OLD.link_type;
            INSERT OR IGNORE INTO sky_links (source_path, target_name, link_type, weight)
            SELECT NEW.source_path, NEW.target_name, NEW.link_type, COALESCE(NEW.weight, 1.0)
            WHERE NEW.status = 'active';
        END;
    ").map_err(|e| format!("Failed to create sky_link triggers: {}", e))?;

    // ─── Sky-node triggers (MIG-001 Step 4) ─────────────────────────────
    // Keep sky_nodes in lock-step with note_meta. Orphans are preserved
    // intrinsically (Invariant 1): a row exists in sky_nodes for every
    // row in note_meta regardless of whether note_links references it,
    // because the trigger fires on note_meta writes directly.
    //
    // link_count / outgoing_count stay at 0 here. They're computed on
    // the read side in Step 8's new IPC (COUNT(*) GROUP BY over
    // sky_links) rather than maintained at write time. Per-link bumps
    // would require cross-table triggers (note_links trigger also
    // updating sky_nodes counters) that complicate the write path for
    // data that a single SQL query can derive at read time cheaply.
    // Enrichment columns (stratum, maturity, origin_type) land in
    // Step 7 via a separate trigger on properties_json changes.
    //
    // Rename cascade (verified in Step 6): the end-to-end chain is more
    // complex than the AU trigger alone. `index_note` writes note_meta
    // via DELETE + INSERT (not UPDATE — see search.rs:966/969), so a
    // rename-driven reindex fires the AD+AI pair here, NOT AU. The AU
    // trigger below exists for the rare case where note_meta is
    // UPDATE'd directly (e.g. future code that edits a single column
    // without a full reindex). It's defensive coverage, not the primary
    // rename path.
    //
    // Actual rename flow for a canonical file (the common case):
    //   1. rename_item updates frontmatter title in-place, calls
    //      reindex_single_note.
    //   2. index_note: DELETE note_meta; INSERT note_meta with new
    //      name. AD fires (sky_nodes row + outgoing sky_links deleted);
    //      AI fires (sky_nodes row recreated with new name).
    //   3. index_note: DELETE note_links; INSERT new note_links rows.
    //      note_links_sky_ad and _ai fire for each, rebuilding outgoing
    //      sky_links.
    //   4. update_links_on_rename walks every other .md file, rewrites
    //      `[[old-name]]` → `[[new-name]]`, calls reindex for each.
    //      Step 2+3 repeat for each affected source, updating
    //      target_name in their sky_links rows via the note_links
    //      trigger chain.
    //
    // During the window between step 2 and step 4 completion, sky_links
    // has stale target_name for incoming edges. Transient; self-heals
    // on UI refresh after update_links_on_rename completes. A single
    // atomic UPDATE would require rewriting the DELETE+INSERT pattern
    // at search.rs:976, which is load-bearing for the preserved-
    // traversal-data snapshot there.
    //
    // The AU WHEN guard limits firing to structural changes
    // (path / name / library_name). Frequent note saves that only
    // touch modified / content_hash / body_text etc. don't cascade
    // into sky_* unnecessarily — that's the typing-latency guardrail
    // (Invariant 8).
    // MIG-002 §6 + §99/BUG-011: drop note_meta_sky_au AND note_meta_sky_ai
    // so their bodies can be rewritten for §6 (UPDATE-preserving AU) and
    // BUG-011 (merged AI with inline stratum + maturity).
    conn.execute_batch("
        DROP TRIGGER IF EXISTS note_meta_sky_au;
        DROP TRIGGER IF EXISTS note_meta_sky_ai;
    ").map_err(|e| format!("Failed to drop pre-§6/§99 sky triggers: {}", e))?;

    conn.execute_batch(&format!("
        -- BUG-011 workaround: inline stratum + maturity computation in
        -- the INSERT trigger body. On this SQLite build, keeping them
        -- as separate AFTER INSERT triggers resulted in silent skipping
        -- — only the first AI trigger (which does INSERT OR REPLACE on
        -- sky_nodes) executed. Merging the sky_nodes INSERT and the
        -- stratum + maturity UPDATE into one trigger body sidesteps
        -- the multi-trigger dispatch issue entirely.
        DROP TRIGGER IF EXISTS note_meta_sky_ai;
        CREATE TRIGGER IF NOT EXISTS note_meta_sky_ai
        AFTER INSERT ON note_meta
        BEGIN
            INSERT OR REPLACE INTO sky_nodes (path, id, name, library_name, cid_cn, updated_at)
            VALUES (NEW.path, LOWER(NEW.name), NEW.name, NEW.library_name, NEW.cid_cn, strftime('%s','now'));
            UPDATE sky_nodes SET stratum = ({stratum_expr}) WHERE path = NEW.path;
            UPDATE sky_nodes SET maturity = ({maturity_expr}) WHERE path = NEW.path;
        END;

        CREATE TRIGGER IF NOT EXISTS note_meta_sky_ad
        AFTER DELETE ON note_meta
        BEGIN
            DELETE FROM sky_nodes WHERE path = OLD.path;
            -- Cascade: outgoing edges sourced from the deleted note
            -- must go too. Incoming edges (where this note was the
            -- target) are left alone — the source notes still exist
            -- and their note_links rows remain valid; the absent
            -- target will resolve as an orphan/red wikilink.
            DELETE FROM sky_links WHERE source_path = OLD.path;
        END;

        -- MIG-002 §6: UPDATE-preserving rename trigger.
        --
        -- The fields that change on a rename (path / name / library_name)
        -- do NOT change the structural inputs to stratum or maturity —
        -- word count, link counts, link types, timestamps all stay put.
        -- So their VALUES are provably unchanged and we preserve them
        -- instead of recomputing (faster + no transient NULL window).
        --
        -- origin_type is preserved too for the renamed note itself,
        -- but any DESCENDANT note (linked to this one via derives-from)
        -- has its origin_type invalidated: when an ancestor renames, the
        -- chain walk that produced the descendant's origin_type now
        -- operates on a changed identity. We stamp enrichment_dirty=1
        -- on those descendants so the §7 background worker recomputes
        -- them on next drain. The EXISTS subquery checks whether this
        -- rename affects any derives-from edge; if not, no stamping is
        -- needed and the maximum-work branch is skipped entirely.
        CREATE TRIGGER IF NOT EXISTS note_meta_sky_au
        AFTER UPDATE ON note_meta
        WHEN OLD.path IS NOT NEW.path
          OR OLD.name IS NOT NEW.name
          OR OLD.library_name IS NOT NEW.library_name
        BEGIN
            -- Rewrite sky_nodes in place. UPDATE preserves stratum,
            -- maturity, origin_type, enrichment_dirty. path is the PK
            -- so a path change updates the row's PK; SQLite allows this
            -- as long as the new path doesn't collide (it won't — notes
            -- have unique paths by filesystem invariant).
            UPDATE sky_nodes
               SET path = NEW.path,
                   id = LOWER(NEW.name),
                   name = NEW.name,
                   library_name = NEW.library_name,
                   updated_at = strftime('%s','now')
             WHERE path = OLD.path;

            -- Migrate edges referencing the old identity. target_name
            -- match uses LOWER(OLD.name) because note_links stores
            -- target_name pre-lowercased (all 232k rows on the target
            -- universe confirm this — see BUG-010).
            UPDATE sky_links
               SET source_path = NEW.path
             WHERE source_path = OLD.path
               AND OLD.path IS NOT NEW.path;
            UPDATE sky_links
               SET target_name = LOWER(NEW.name)
             WHERE target_name = LOWER(OLD.name)
               AND LOWER(OLD.name) IS NOT LOWER(NEW.name);

            -- MIG-004 §4: cascade alias rows on path change. Only
            -- triggered when path actually moves — a name-only or
            -- library-only rename leaves the path PK untouched, so
            -- alias rows for the original path are still correct.
            -- Note: this AU path is rare in practice (index_note uses
            -- DELETE+INSERT, which fires AD+AI not AU); the cascade is
            -- defensive coverage for direct-UPDATE writers (test code,
            -- future migrations).
            --
            -- Deliberately NOT extending note_meta_sky_ad to also
            -- DELETE alias rows — that AD fires on every save's
            -- DELETE+INSERT cycle and would clobber 'rename' / 'import'
            -- rows that §3 worked to make durable. Orphaned alias rows
            -- (path no longer in note_meta) are harmless: nothing
            -- JOINs to them, queries that filter by path stay correct.
            UPDATE note_aliases
               SET path = NEW.path
             WHERE path = OLD.path
               AND OLD.path IS NOT NEW.path;

            -- Conditional origin_type dirty cascade. Scoped to the set
            -- of notes affected by a derives-from edge touching the
            -- renamed note:
            --   (a) this note itself, if it has a derives-from edge in
            --       or out (self's ancestry may resolve differently now)
            --   (b) descendants: notes that link to this one with
            --       link_type='derives-from' (they walk their chain
            --       through this node, which just changed identity)
            -- The OR-split on the WHERE keeps the subqueries narrow.
            -- enrichment_dirty=1 is idempotent; if already 1, no-op.
            UPDATE sky_nodes SET enrichment_dirty = 1
             WHERE path = NEW.path
               AND EXISTS(
                   SELECT 1 FROM note_links
                    WHERE (source_path = NEW.path OR source_path = OLD.path)
                      AND link_type = 'derives-from'
                      AND status = 'active');
            UPDATE sky_nodes SET enrichment_dirty = 1
             WHERE path IN (
                   SELECT source_path FROM note_links
                    WHERE (target_name = LOWER(OLD.name) OR target_name = LOWER(NEW.name))
                      AND link_type = 'derives-from'
                      AND status = 'active');
        END;
    ", stratum_expr = STRATUM_SQL_EXPR, maturity_expr = MATURITY_SQL_EXPR))
    .map_err(|e| format!("Failed to create sky_node triggers: {}", e))?;

    // ─── MIG-002 §4: SQL-native stratum triggers ────────────────────────
    //
    // Stratum (1–8) is a function of five signals all derivable in SQL:
    //   base        — note_meta.word_count bucket
    //   +1          — outgoing link count ≥ 3
    //   +1          — inbound link count ≥ 5
    //   +1          — outgoing has a 'generalizes' edge
    //   +1          — outgoing has a 'causes' or 'supports' edge
    //   +1          — distinct inbound source count ≥ 3
    //
    // Identical to strata.rs::compute_stratum. Triggers keep the value
    // fresh on every write to note_meta (body / word_count changes) and
    // note_links (edge changes affecting source or target).
    //
    // Expression is correlated on sky_nodes.path / sky_nodes.id (NOT
    // .name — note_links.target_name is stored lowercase, and sky_nodes.id
    // is LOWER(name). BUG-010 caught this: the v4 formula matched on
    // .name and got 0 inbound for every non-lowercase note). Shared via
    // STRATUM_SQL_EXPR with the one-shot back-fill in sky_backfill.rs —
    // single source of truth.
    //
    // DROP first so schema-version bumps that change the trigger body
    // (like v4 → v5 for BUG-010) pick up the new formula. CREATE TRIGGER
    // IF NOT EXISTS alone would silently keep the old body on upgrade.
    conn.execute_batch("
        DROP TRIGGER IF EXISTS note_meta_sky_stratum_ai;
        DROP TRIGGER IF EXISTS note_meta_sky_stratum_au;
        DROP TRIGGER IF EXISTS note_links_sky_stratum_ai;
        DROP TRIGGER IF EXISTS note_links_sky_stratum_ad;
        DROP TRIGGER IF EXISTS note_links_sky_stratum_au;
    ").map_err(|e| format!("Failed to drop old stratum triggers: {}", e))?;

    // Drop diagnostic + any prior separate stratum AI / AU triggers.
    // BUG-011 investigation uncovered that on this SQLite build,
    // multiple separate AFTER INSERT triggers on note_meta silently
    // skip the later ones in the chain once an earlier trigger body
    // has written to another table (recursive_triggers=ON did not
    // help). Workaround: inline the stratum + maturity UPDATE into
    // the existing MIG-001 note_meta_sky_ai body so it's one trigger,
    // one body, no multi-trigger dispatch.
    conn.execute_batch("
        DROP TRIGGER IF EXISTS note_meta_sky_stratum_ai_DIAG;
        DROP TRIGGER IF EXISTS note_meta_sky_stratum_ai;
        DROP TRIGGER IF EXISTS note_meta_sky_stratum_au;
    ").map_err(|e| format!("drop AI/AU legacy stratum: {}", e))?;

    conn.execute_batch(&format!("
        -- note_meta update: recompute only when word_count actually
        -- changes. AU triggers don't seem to have the same skip issue
        -- as AI chains (§6's note_meta_sky_au is the sole AU writer
        -- to sky_nodes). Keeping this AU as a separate trigger.
        CREATE TRIGGER IF NOT EXISTS note_meta_sky_stratum_au
        AFTER UPDATE ON note_meta
        WHEN NEW.word_count IS NOT OLD.word_count
        BEGIN
            UPDATE sky_nodes SET stratum = ({expr}) WHERE path = NEW.path;
        END;

        -- note_links insert: new active edge changes source's outgoing
        -- count + target's inbound count. Archived links (status != active)
        -- don't contribute to the stratum formula — skip the trigger body
        -- via WHEN. target_name match updates ALL sky_nodes sharing that
        -- lowercased name (expected behavior — inbound_count is name-
        -- scoped in strata.rs).
        CREATE TRIGGER IF NOT EXISTS note_links_sky_stratum_ai
        AFTER INSERT ON note_links
        WHEN NEW.status = 'active'
        BEGIN
            UPDATE sky_nodes SET stratum = ({expr}) WHERE path = NEW.source_path;
            UPDATE sky_nodes SET stratum = ({expr}) WHERE id = NEW.target_name;
        END;

        -- note_links delete: symmetric to insert — the lost edge changes
        -- source's outgoing count + target's inbound count.
        CREATE TRIGGER IF NOT EXISTS note_links_sky_stratum_ad
        AFTER DELETE ON note_links
        WHEN OLD.status = 'active'
        BEGIN
            UPDATE sky_nodes SET stratum = ({expr}) WHERE path = OLD.source_path;
            UPDATE sky_nodes SET stratum = ({expr}) WHERE id = OLD.target_name;
        END;

        -- note_links update: covers re-type (link_type changed), archive
        -- toggle (status changed), and rename cascade (source_path or
        -- target_name changed via §6). Touches both OLD and NEW identities
        -- so stratum is correct for both sides.
        CREATE TRIGGER IF NOT EXISTS note_links_sky_stratum_au
        AFTER UPDATE ON note_links
        WHEN OLD.status IS NOT NEW.status
          OR OLD.link_type IS NOT NEW.link_type
          OR OLD.source_path IS NOT NEW.source_path
          OR OLD.target_name IS NOT NEW.target_name
        BEGIN
            UPDATE sky_nodes SET stratum = ({expr}) WHERE path = OLD.source_path;
            UPDATE sky_nodes SET stratum = ({expr}) WHERE path = NEW.source_path;
            UPDATE sky_nodes SET stratum = ({expr}) WHERE id = OLD.target_name;
            UPDATE sky_nodes SET stratum = ({expr}) WHERE id = NEW.target_name;
        END;
    ", expr = STRATUM_SQL_EXPR))
    .map_err(|e| format!("Failed to create stratum triggers: {}", e))?;

    // ─── MIG-002 §5: SQL-native maturity triggers ───────────────────────
    //
    // Mirrors §4 stratum shape. Five triggers keep sky_nodes.maturity in
    // sync with the three signals that drive it: inbound count,
    // days_since_created, days_since_modified. See MATURITY_SQL_EXPR for
    // the CASE chain that maps those to seed / sapling / evergreen /
    // canonical / wilting.
    //
    // DROP first (same pattern as §4 / §96) so formula changes on
    // version bumps pick up the new body.
    conn.execute_batch("
        DROP TRIGGER IF EXISTS note_meta_sky_maturity_ai;
        DROP TRIGGER IF EXISTS note_meta_sky_maturity_au;
        DROP TRIGGER IF EXISTS note_links_sky_maturity_ai;
        DROP TRIGGER IF EXISTS note_links_sky_maturity_ad;
        DROP TRIGGER IF EXISTS note_links_sky_maturity_au;
    ").map_err(|e| format!("Failed to drop old maturity triggers: {}", e))?;

    conn.execute_batch(&format!("
        -- note_meta insert: maturity AI is INLINED into note_meta_sky_ai
        -- above (BUG-011 workaround — multiple AFTER INSERT triggers on
        -- note_meta don't all fire on this SQLite build). The separate
        -- trigger is intentionally NOT recreated here.

        -- note_meta update: recompute when `modified` or `created_at`
        -- changes. `modified` changes on every save by definition, so
        -- this trigger fires with every note edit — cheap (one UPDATE
        -- of one row using the maturity CASE chain).
        CREATE TRIGGER IF NOT EXISTS note_meta_sky_maturity_au
        AFTER UPDATE ON note_meta
        WHEN NEW.modified IS NOT OLD.modified
          OR NEW.created_at IS NOT OLD.created_at
        BEGIN
            UPDATE sky_nodes SET maturity = ({expr}) WHERE path = NEW.path;
        END;

        -- note_links insert: a new active edge changes the target's
        -- inbound count → recompute target's maturity. Source maturity
        -- does NOT depend on outgoing edges; skip the source update.
        CREATE TRIGGER IF NOT EXISTS note_links_sky_maturity_ai
        AFTER INSERT ON note_links
        WHEN NEW.status = 'active'
        BEGIN
            UPDATE sky_nodes SET maturity = ({expr}) WHERE id = NEW.target_name;
        END;

        -- note_links delete: symmetric — deletion drops target's inbound.
        CREATE TRIGGER IF NOT EXISTS note_links_sky_maturity_ad
        AFTER DELETE ON note_links
        WHEN OLD.status = 'active'
        BEGIN
            UPDATE sky_nodes SET maturity = ({expr}) WHERE id = OLD.target_name;
        END;

        -- note_links update: covers archive toggle (status) and rename
        -- cascade (target_name changed via §6). Only target-side is
        -- relevant for maturity; no source updates. Both OLD and NEW
        -- target identities are touched.
        CREATE TRIGGER IF NOT EXISTS note_links_sky_maturity_au
        AFTER UPDATE ON note_links
        WHEN OLD.status IS NOT NEW.status
          OR OLD.target_name IS NOT NEW.target_name
        BEGIN
            UPDATE sky_nodes SET maturity = ({expr}) WHERE id = OLD.target_name;
            UPDATE sky_nodes SET maturity = ({expr}) WHERE id = NEW.target_name;
        END;
    ", expr = MATURITY_SQL_EXPR))
    .map_err(|e| format!("Failed to create maturity triggers: {}", e))?;

    // The one-shot stratum + maturity back-fill for existing sky_nodes
    // rows runs in sky_backfill.rs on a background thread — NOT here.
    // Same reasons as §4: boot-paint-blocking cost + reuse of the
    // resumable cursor. Bumping SKY_SCHEMA_VERSION (now v6) forces
    // sky_backfill::maybe_schedule to re-run.

    // ─── MIG-018 §1A: Sight v3 projection-foundation cache tables ────
    //
    // Three tables store the deterministic Landmark-MDS embedding for the
    // v3 star-chart visualization (PJ-038). Per CLAUDE.md Performance
    // Rule 8 (write-time derivation): the cache is invalidated by triggers
    // on `note_links` writes; reads at frontend Sight-toggle are cheap
    // SELECTs.
    //
    //   sight_v3_layout         — one row per (note_path, lib_set, ver):
    //                             cached embed_x, embed_y in unit-disk
    //                             coords + community_id + centrality_norm.
    //   sight_v3_layout_cursor  — resumable back-fill cursor, mirrors
    //                             sky_backfill_cursor (MIG-001 §5).
    //   sight_v3_graph_version  — single-row meta per library_set_hash;
    //                             bumped by note_links triggers; the
    //                             frontend compares cached vs current
    //                             version to decide recompute.
    //
    // Trigger semantics: any insert/delete on note_links bumps the version
    // for every library_set_hash. (We can't know which library_set_hash
    // each link affects without joining note_meta.library_name; the
    // frontend will recompute lazily on next toggle, which is acceptable
    // — embedding is sub-second on Boss-scale graphs.)
    //
    // Schema is gated by SIGHT_V3_SCHEMA_VERSION; bumping it invalidates
    // every cached layout. The frontend must observe the bump (via the
    // `constellation_sight_v3_layout` IPC return value) and recompute.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS sight_v3_layout (
            note_path        TEXT NOT NULL,
            library_set_hash TEXT NOT NULL,
            graph_version    INTEGER NOT NULL,
            embed_x          REAL NOT NULL,
            embed_y          REAL NOT NULL,
            community_id     INTEGER NOT NULL,
            centrality_norm  REAL NOT NULL,
            PRIMARY KEY (note_path, library_set_hash, graph_version)
        );
        CREATE INDEX IF NOT EXISTS idx_sight_v3_layout_libset_ver
            ON sight_v3_layout(library_set_hash, graph_version);

        CREATE TABLE IF NOT EXISTS sight_v3_layout_cursor (
            library_set_hash TEXT PRIMARY KEY,
            graph_version    INTEGER NOT NULL,
            completed        INTEGER NOT NULL DEFAULT 0,
            started_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            completed_at     INTEGER
        );

        CREATE TABLE IF NOT EXISTS sight_v3_graph_version (
            library_set_hash TEXT PRIMARY KEY,
            version          INTEGER NOT NULL DEFAULT 0,
            bumped_at        INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );

        -- Note: sight_v3_graph_version bumps live in the application code,
        -- not in SQL triggers. Reason: a SQL trigger on note_links would
        -- need to know the library_set_hash that the changed note belongs
        -- to, which requires joining note_meta + libraries config — too
        -- expensive to do per-row inside a trigger. Instead, the frontend
        -- bumps on graph-version writes via a deliberate IPC call after
        -- batch operations, mirroring v2's lensDataStale invalidation.
        --
        -- For MIG-018 §1A: tables-only. Bump-on-edit semantics land in
        -- §1B alongside the compute path.

        -- MIG-019 §2A+§2B redesign: TF-IDF content-similarity DENSITY GRID
        -- (PJ-035 → Milky Way). Replaces the v2 sight_v3_similarity_edges
        -- table after Eisa's 2026-05-07 directive ('Don't patch it. Solve
        -- it.') exposed the edge-list approach as fundamentally OOM-prone:
        -- accumulating candidate pairs as cloned-string SimilarityEdge
        -- structs allocates O(candidate_pairs × ~200 bytes) Rust heap
        -- BEFORE the output cap fires — sub-Gigabyte universes were
        -- crashing the Rust process on the all-pairs accumulator.
        --
        -- The density-grid architecture per Concept Paper v1.1 §5.1:
        -- one row per (library_set_hash, graph_version) holds a 256×256
        -- f32 grid as a BLOB (256² × 4 = 262,144 bytes ≈ 256 KB).
        -- Memory is now bounded by OUTPUT, not INPUT: each candidate
        -- pair above the similarity threshold rasterizes a line into the
        -- grid (DDA, ~100 cells per pair) and is then dropped. The grid
        -- accumulates similarity weights; a Gaussian blur smooths it
        -- into the diffuse band texture. Universe size becomes
        -- irrelevant to memory pressure.
        CREATE TABLE IF NOT EXISTS sight_v3_density_grid (
            library_set_hash  TEXT NOT NULL,
            graph_version     INTEGER NOT NULL,
            width             INTEGER NOT NULL,
            height            INTEGER NOT NULL,
            max_value         REAL NOT NULL,
            data              BLOB NOT NULL,
            computed_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            PRIMARY KEY (library_set_hash, graph_version)
        );

        -- MIG-019 v3 redesign: drop the v2 edge-list table on schema
        -- bump. Idempotent — DROP IF EXISTS is a no-op on fresh DBs.
        DROP TABLE IF EXISTS sight_v3_similarity_edges;
    ").map_err(|e| format!("Failed to create sight_v3_* tables: {}", e))?;

    // ── MIG-019 §2A: SIGHT_V3_SCHEMA_VERSION cache invalidation ────
    //
    // If the stored sight_v3 module version is below the current target,
    // wipe all four cache tables so the next user-driven Sight v3 toggle
    // does a cold compute against the new schema. The wipe is idempotent
    // and runs after the CREATE TABLE IF NOT EXISTS statements above
    // (so empty rows on a fresh DB don't cause an error).
    //
    // Eisa-approved 2026-05-07 (Architect §5 row 2): "acceptable cosmetic
    // cache pollution; only happens once per upgrade." Sub-second cold
    // compute on Boss-scale graphs.
    let stored_sight_v3_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'sight_v3'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stored_sight_v3_version < SIGHT_V3_SCHEMA_VERSION {
        diag_log(path, &format!(
            "[search] init_db: schema_versions.sight_v3={} (target {}) — wiping cache tables for cold recompute",
            stored_sight_v3_version, SIGHT_V3_SCHEMA_VERSION,
        ));
        conn.execute_batch("
            DELETE FROM sight_v3_layout;
            DELETE FROM sight_v3_layout_cursor;
            DELETE FROM sight_v3_graph_version;
            DELETE FROM sight_v3_density_grid;
        ").map_err(|e| format!("Failed to wipe sight_v3 cache for version bump: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
             VALUES ('sight_v3', ?1, strftime('%s','now'))",
            rusqlite::params![SIGHT_V3_SCHEMA_VERSION],
        ).map_err(|e| format!("Failed to stamp schema_versions.sight_v3: {}", e))?;
    }

    // ─── One-time FTS5 rebuild after tokenizer migration ─────────────
    // If we bumped past FTS_SCHEMA_VERSION above we dropped the old
    // `notes_fts` + `notes_vocab`. The `CREATE VIRTUAL TABLE IF NOT
    // EXISTS` statements above re-created them with the new tokenizer,
    // but empty — there's no content yet. `INSERT INTO notes_fts(notes_fts)
    // VALUES('rebuild')` walks the content table (`note_meta`) and
    // re-tokenizes every row through our custom pipeline, populating
    // the inverted index.
    //
    // This happens inline in `init_db`, which is called once per
    // Universe open. For the 7,600-note trial Universe this is expected
    // to complete in well under 10 seconds (FTS5 reads the content
    // table sequentially; our tokenizer is pure Rust stemming). If
    // measurement shows it above that threshold we'll move the rebuild
    // to a background task post-paint, per Rule 8's first-time
    // population guidance.
    //
    // A `wal_checkpoint(TRUNCATE)` afterwards prevents the large
    // transaction from bloating the WAL (learned the hard way — a
    // previous aborted streaming run left a 3.1 GB WAL that froze boot).
    if needs_fts_rebuild {
        let rebuild_start = std::time::Instant::now();
        conn.execute_batch("INSERT INTO notes_fts(notes_fts) VALUES('rebuild');")
            .map_err(|e| format!("Failed to rebuild notes_fts: {}", e))?;
        let rebuild_ms = rebuild_start.elapsed().as_millis();

        // Stamp the new schema version BEFORE checkpoint so that a crash
        // after checkpoint but before PRAGMA wouldn't trigger a spurious
        // second rebuild.
        conn.execute_batch(&format!("PRAGMA user_version = {};", FTS_SCHEMA_VERSION))
            .map_err(|e| format!("Failed to stamp user_version: {}", e))?;

        // Truncate WAL so the large rebuild transaction doesn't haunt
        // future boots. Ignore errors — this is hygiene, not correctness.
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");

        diag_log(path, &format!(
            "[search] notes_fts rebuilt with 'constellation' tokenizer in {} ms",
            rebuild_ms
        ));
    }

    // ─── MIG-003 Step 2 — cid_cn columns on dependent tables ────────────
    // Idempotent ALTERs first; then a one-shot back-fill gated on
    // schema_versions.dependent_tables_mig003. Triggers that maintain
    // these columns on subsequent writes land in Step 3.
    ensure_note_links_mig003_columns(&conn)
        .map_err(|e| format!("Failed to ensure note_links MIG-003 columns: {}", e))?;
    ensure_sky_nodes_mig003_columns(&conn)
        .map_err(|e| format!("Failed to ensure sky_nodes MIG-003 column: {}", e))?;
    ensure_note_aliases_mig003_columns(&conn)
        .map_err(|e| format!("Failed to ensure note_aliases MIG-003 column: {}", e))?;
    ensure_note_embeddings_mig003_columns(&conn)
        .map_err(|e| format!("Failed to ensure note_embeddings MIG-003 column: {}", e))?;
    let stored_dep_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'dependent_tables_mig003'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stored_dep_version < DEPENDENT_TABLES_MIG003_VERSION {
        diag_log(path, &format!(
            "[search] init_db: schema_versions.dependent_tables_mig003={} (target {}) — Step 2 backfill needed",
            stored_dep_version,
            DEPENDENT_TABLES_MIG003_VERSION,
        ));
        mig003_step2_backfill(&mut conn, path)
            .map_err(|e| format!("Failed to backfill cid_cn on dependent tables: {}", e))?;
        ensure_dependent_tables_mig003_indexes(&conn)
            .map_err(|e| format!("Failed to add cid_cn indexes on dependent tables: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('dependent_tables_mig003', ?1, strftime('%s','now'))",
            rusqlite::params![DEPENDENT_TABLES_MIG003_VERSION],
        ).map_err(|e| format!("Failed to stamp schema_versions.dependent_tables_mig003: {}", e))?;
        diag_log(path, &format!(
            "[search] init_db: schema_versions.dependent_tables_mig003 stamped to {} after MIG-003 Step 2 backfill",
            DEPENDENT_TABLES_MIG003_VERSION,
        ));
    } else {
        diag_log(path, &format!(
            "[search] init_db: schema_versions.dependent_tables_mig003={} (target {}) — Step 2 backfill skipped (already done)",
            stored_dep_version,
            DEPENDENT_TABLES_MIG003_VERSION,
        ));
    }

    // MIG-003 Step 3 — boot-time soft re-backfill. Repairs any cid_cn
    // hole left by a writer that didn't include cid_cn (e.g. external
    // sync drop, mid-flight indexer interruption). Cheap when nothing
    // to fix; logs only when it actually repairs rows.
    let _ = mig003_step3_soft_rebackfill(&mut conn, path);

    // MIG-003 Step 4 — canonical → human filename migration. One-shot,
    // gated by schema_versions.mig003_step4. Walks every library,
    // renames every canonical-named .md to its title-derived name,
    // cascades the path change to every dependent table inside a
    // per-library transaction. Audit log written to
    // .constellation/mig003-step4-renames.tsv. Idempotent on restart.
    let stored_step4_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_versions WHERE module = 'mig003_step4'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stored_step4_version < crate::mig003_step4::MIG003_STEP4_VERSION {
        diag_log(path, &format!(
            "[search] init_db: schema_versions.mig003_step4={} (target {}) — Step 4 rename pass starting",
            stored_step4_version,
            crate::mig003_step4::MIG003_STEP4_VERSION,
        ));
        match crate::mig003_step4::run(&mut conn, path) {
            Ok(()) => {
                conn.execute(
                    "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('mig003_step4', ?1, strftime('%s','now'))",
                    rusqlite::params![crate::mig003_step4::MIG003_STEP4_VERSION],
                ).map_err(|e| format!("Failed to stamp schema_versions.mig003_step4: {}", e))?;
                diag_log(path, &format!(
                    "[search] init_db: schema_versions.mig003_step4 stamped to {} after Step 4 rename pass",
                    crate::mig003_step4::MIG003_STEP4_VERSION,
                ));
            }
            Err(e) => {
                diag_log(path, &format!(
                    "[search] init_db: MIG-003 Step 4 returned an error (no stamp; will retry next boot): {}",
                    e,
                ));
            }
        }
    } else {
        diag_log(path, &format!(
            "[search] init_db: schema_versions.mig003_step4={} (target {}) — Step 4 already done",
            stored_step4_version,
            crate::mig003_step4::MIG003_STEP4_VERSION,
        ));
    }

    Ok(conn)
}

// ─── Indexing Pipeline ─────────────────────────────────────────

/// Locate the body slice of a note — everything after the closing
/// `---` of the YAML frontmatter, or the full content if there is no
/// frontmatter block. Zero-copy. Shared by `parse_frontmatter` (which
/// needs to parse properties/tags from the frontmatter) and
/// `sky_backfill::compute_word_count_and_created_at` (which only needs
/// the body for word counting). Single source of truth for the strip
/// shape so back-fill and writer agree byte-for-byte.
pub(crate) fn body_after_frontmatter(content: &str) -> &str {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            return &content[3 + end + 3..];
        }
    }
    content
}

/// Parse frontmatter properties from YAML block.
fn parse_frontmatter(content: &str) -> (HashMap<String, String>, Vec<String>, String) {
    let mut properties = HashMap::new();
    let mut tags = Vec::new();
    let mut body = content.to_string();

    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let fm = &content[3..3 + end];
            body = body_after_frontmatter(content).trim().to_string();

            let mut in_tags = false;
            for line in fm.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("tags:") {
                    in_tags = true;
                    // Inline tags: tags: [a, b] or tags: a, b
                    let val = trimmed[5..].trim();
                    if !val.is_empty() {
                        // Strip brackets for [a, b] format
                        let val = val.trim_start_matches('[').trim_end_matches(']');
                        for t in val.split(',') {
                            let t = t.trim().trim_matches(|c| c == '"' || c == '\'');
                            if !t.is_empty() { tags.push(t.to_lowercase()); }
                        }
                    }
                    continue;
                }
                if in_tags {
                    if trimmed.starts_with("- ") {
                        let tag = trimmed[2..].trim().to_lowercase();
                        if !tag.is_empty() { tags.push(tag); }
                    } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        in_tags = false;
                    }
                }
                if !in_tags && trimmed.contains(':') && !trimmed.starts_with('#') {
                    if let Some(idx) = trimmed.find(':') {
                        let key = trimmed[..idx].trim().to_string();
                        let val = trimmed[idx + 1..].trim().trim_matches('"').to_string();
                        if !key.is_empty() { properties.insert(key, val); }
                    }
                }
            }
        }
    }

    // Also extract inline #hashtags from body text
    let tag_re = regex::Regex::new(r"(?:^|\s)#([\w\p{L}\p{N}_/-]+)").unwrap();
    for cap in tag_re.captures_iter(&body) {
        if let Some(m) = cap.get(1) {
            let tag = m.as_str().trim().to_lowercase();
            if !tag.is_empty() && !tags.contains(&tag) {
                tags.push(tag);
            }
        }
    }

    (properties, tags, body)
}

/// Extract outgoing wikilinks from note content.
/// Applies Arabic normalization for consistent matching with title-based names.
fn extract_wikilinks(content: &str) -> Vec<String> {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]").unwrap());
    let mut links = Vec::new();
    for cap in re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let target = normalize_arabic_for_search(&m.as_str().trim().to_lowercase());
            if !target.is_empty() && !links.contains(&target) {
                links.push(target);
            }
        }
    }
    links
}

/// MIG-004 §2: extract YAML `aliases:` from a note's frontmatter.
///
/// Handles all three shapes Constellation accepts:
///
/// ```yaml
/// aliases: foo                  # scalar
/// aliases: [foo, bar]           # inline array
/// aliases:                      # YAML list
///   - foo
///   - bar
/// ```
///
/// Each alias goes through the same normalization as `extract_wikilinks`
/// (lowercase + Arabic) so the resulting `alias_lower` values match
/// `note_links.target_name` byte-for-byte. That's what makes the
/// alias-aware inbound JOINs in MIG-004 §6/§7 a direct equality.
///
/// Block-aware: tracks "are we currently inside the `aliases:` list
/// block" so a `-` line item that follows `tags:` or another list
/// field is NOT mistakenly consumed as an alias. The pre-existing
/// `libraries.rs::has_alias` lacks this guard; this implementation
/// fixes that latent bug.
pub(crate) fn extract_aliases(content: &str) -> Vec<String> {
    if !content.starts_with("---") {
        return Vec::new();
    }
    let Some(end) = content[3..].find("\n---") else {
        return Vec::new();
    };
    let frontmatter = &content[3..3 + end];

    let mut aliases: Vec<String> = Vec::new();
    let mut in_aliases_block = false;

    for line in frontmatter.lines() {
        let trimmed = line.trim_start();

        // `aliases:` opener — handles inline and block forms.
        if trimmed.starts_with("aliases:") {
            in_aliases_block = true;
            let value = trimmed["aliases:".len()..].trim();
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                for raw in inner.split(',') {
                    push_alias(&mut aliases, raw);
                }
                in_aliases_block = false;
            } else if !value.is_empty() {
                push_alias(&mut aliases, value);
                in_aliases_block = false;
            }
            continue;
        }

        // While inside the block, accept `- value` items.
        if in_aliases_block {
            if let Some(rest) = trimmed.strip_prefix("- ") {
                push_alias(&mut aliases, rest);
                continue;
            }
            // Any non-list-item line ends the block (next field, blank, etc.).
            if !trimmed.is_empty() {
                in_aliases_block = false;
            }
        }
    }
    aliases
}

fn push_alias(out: &mut Vec<String>, raw: &str) {
    let normalized = normalize_alias_for_match(raw);
    if !normalized.is_empty() && !out.contains(&normalized) {
        out.push(normalized);
    }
}

/// MIG-004 §3 helper — normalize an alias string for storage in
/// `note_aliases.alias_lower` so it matches `note_links.target_name`
/// byte-for-byte. Trim quoting, lowercase, Arabic-normalize. Empty
/// returns empty (caller's job to skip empties).
///
/// Exposed pub(crate) so the rename writer in `libraries.rs` can
/// produce the same byte representation when stamping a 'rename'
/// alias row, without re-implementing the trim+normalize chain.
pub(crate) fn normalize_alias_for_match(raw: &str) -> String {
    let cleaned = raw
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim();
    if cleaned.is_empty() {
        return String::new();
    }
    normalize_arabic_for_search(&cleaned.to_lowercase())
}

/// A typed link extracted from note content.
#[derive(Debug, Clone)]
struct TypedLink {
    target: String,       // target note name (lowercase)
    link_type: String,    // supports, contradicts, causes, etc.
    annotation: String,   // user's reasoning (from |annotation syntax)
}

/// Extract typed links from note content.
/// Matches: [[type::target|annotation]], [[type::target]], [[target|display]], [[target]]
fn extract_typed_links(content: &str) -> Vec<TypedLink> {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    // Matches: [[optional_type::target|optional_annotation]]
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"\[\[(?:([a-zA-Z\-]+)::)?([^\]|]+?)(?:\|([^\]]*))?\]\]").unwrap()
    });
    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for cap in re.captures_iter(content) {
        let link_type = cap.get(1)
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_else(|| "relates".to_string());
        let target = cap.get(2)
            .map(|m| m.as_str().trim().to_lowercase())
            .unwrap_or_default();
        let annotation = cap.get(3)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        if target.is_empty() { continue; }
        let key = format!("{}::{}", link_type, target);
        if !seen.insert(key) { continue; }

        links.push(TypedLink { target, link_type, annotation });
    }
    links
}

/// Extract headings from markdown content.
fn extract_headings(content: &str) -> Vec<String> {
    let mut headings = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim().to_string();
            if !heading.is_empty() { headings.push(heading); }
        }
    }
    headings
}

/// Arabic text normalization for FTS storage and query matching.
///
/// **Only strips tashkeel (diacritics) and tatweel** — the two
/// unconditionally-safe transformations that every Arabic reader
/// and every dictionary agree do not change the word's identity.
///
/// **Does NOT fold ة/ه, ى/ي, or alif variants.** Those distinctions
/// carry orthographic *and semantic* weight in Modern Standard Arabic —
/// they distinguish genuinely different words, not just spelling
/// variants of one word.
///
/// The canonical motivating pair:
///
/// | Surface | Reading | Meaning                             |
/// |---------|---------|-------------------------------------|
/// | `عبرة`  | ʿibrah  | a lesson / moral ("عبرة لمن اعتبر") |
/// | `عبره`  | ʿabarah | he crossed it / went through it     |
///
/// Different roots, different morphology, different pronunciation,
/// different meaning. Folding ة → ه merges these into a single FTS
/// token — so a search for "عبرة" (a lesson) returns every note that
/// said "مرّ عبره" (he crossed it), and vice versa. That is a
/// **semantic break**, not a cosmetic one. Similarly:
/// - `خليفة` (a caliph) vs `خليفه` (a misspelling — or, parsed as a
///   verb form, "he succeeded him")
/// - `موسى` (Moses — terminal alif maqsura is the correct spelling)
/// - `إسلام`, `آمنة`, `أحمد` (hamza-bearing alifs are part of the word)
///
/// An earlier revision of this helper folded all of the above —
/// violating Constellation's "Language-First by Design" principle
/// (CLAUDE.md) and silently disagreeing with
/// `arabic::normalizer::normalize().stripped` (the canonical stripping
/// used by the override store's key function), so user-authored
/// overrides whose surface contained any of those characters never
/// fired on the FTS path.
///
/// **Trade-off**: misspelled queries ("خليفه" when they meant
/// "خليفة") no longer cross-match. The correct place to handle that
/// is a dedicated query-side spelling-tolerance layer with
/// contextual disambiguation — not lossy transformation at index
/// time that would equally destroy the `عبرة` / `عبره` distinction.
/// Tracked as the open SESSION-LOG follow-up "M8e: spelling-tolerance
/// query layer".
///
/// This function delegates to `arabic::normalizer::normalize_stripped`
/// so there is exactly one tashkeel/tatweel implementation in the
/// codebase — any future range fix benefits every caller at once.
fn normalize_arabic_for_search(text: &str) -> String {
    crate::arabic::normalizer::normalize_stripped(text)
}

/// Strip markdown syntax for plain-text indexing.
/// Pre-compiled regex patterns for strip_markdown (compiled once, reused on every call).
fn strip_md_patterns() -> &'static [regex::Regex; 4] {
    use std::sync::OnceLock;
    static PATTERNS: OnceLock<[regex::Regex; 4]> = OnceLock::new();
    PATTERNS.get_or_init(|| [
        regex::Regex::new(r"(?s)```.*?```").unwrap(),               // code blocks
        regex::Regex::new(r"`[^`]+`").unwrap(),                     // inline code
        regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").unwrap(), // wikilinks
        regex::Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap(),      // markdown links
    ])
}

fn strip_markdown(text: &str) -> String {
    let patterns = strip_md_patterns();
    let mut result = text.to_string();
    result = patterns[0].replace_all(&result, " ").to_string();
    result = patterns[1].replace_all(&result, " ").to_string();
    result = patterns[2].replace_all(&result, "$1").to_string();
    result = patterns[3].replace_all(&result, "$1").to_string();
    result = result.replace('#', " ");
    result = result.replace("**", " ").replace("__", " ").replace('*', " ").replace('_', " ");
    result
}

/// Index a single note into the database.
fn index_note(conn: &Connection, note_path: &str, library_name: &str) -> Result<(), String> {
    let path = Path::new(note_path);
    if !path.exists() || path.extension().map(|e| e != "md").unwrap_or(true) {
        return Ok(());
    }

    let file_stem = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // PERF: mtime-first gate. Previously we read the file into memory THEN
    // checked the cache — meaning every unchanged file was still read from
    // disk on every boot. On a 7,600-note Universe that's 7,600 wasted reads.
    // Now: stat the file, compare to cached mtime, read content only if stale.
    let modified = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);

    let existing_mod: Option<u64> = conn.query_row(
        "SELECT modified FROM note_meta WHERE path = ?1",
        params![note_path],
        |row| row.get(0),
    ).ok();

    if existing_mod == Some(modified) {
        return Ok(()); // Cache hit — no disk read needed.
    }

    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    let (properties, tags, body) = parse_frontmatter(&content);
    let wikilinks = extract_wikilinks(&content);
    let headings = extract_headings(&content);
    let plain_body = strip_markdown(&body);

    // Use frontmatter `title:` as the display name when available (supports canonical filenames).
    // Falls back to file stem for legacy (human-named) files.
    let name = properties.get("title")
        .filter(|t| !t.is_empty())
        .cloned()
        .unwrap_or_else(|| file_stem.clone());

    // MIG-003 Step 3: cid_cn from frontmatter — already injected by
    // canonical::ensure_cid_cn during note creation. Falls back to ''
    // (the schema default) for any legacy file that escaped the
    // backfill; the boot-time soft re-backfill in init_db will repair
    // such rows on the next launch.
    let cid_cn = properties.get("cid_cn")
        .cloned()
        .unwrap_or_default();

    // Arabic normalization for FTS body text.
    //
    // **Tashkeel + tatweel only** — we no longer fold ة/ه, ى/ي, or
    // alif variants. Those distinctions carry meaning in MSA; folding
    // them silently conflates `خليفة` and the `خليفه` misspelling in
    // the index, and disagreed with the override store's key
    // normalizer (which never folds). Misspelling tolerance belongs
    // in a separate spell-check query layer, not in lossy
    // normalization at index time.
    //
    // NOTE: `name` is stored ORIGINAL (not even tashkeel-stripped) so
    // it still matches graph node IDs. Name-side Arabic normalization
    // happens at query time instead.
    let plain_body = normalize_arabic_for_search(&plain_body);

    let props_json = serde_json::to_string(&properties).unwrap_or_default();
    let tags_json = serde_json::to_string(&tags).unwrap_or_default();
    let links_json = serde_json::to_string(&wikilinks).unwrap_or_default();
    let headings_json = serde_json::to_string(&headings).unwrap_or_default();

    // MIG-002: denormalize signals for SQL-native stratum/maturity triggers.
    // word_count uses the frontmatter-stripped body (matches strata.rs
    // semantics — count whitespace-separated tokens including markdown
    // syntax). created_at falls back to `modified` when the filesystem
    // lacks a true creation timestamp (ReFS, FAT32, some Linux FS).
    let word_count = body.split_whitespace().count() as i64;
    let created_at: i64 = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(modified as i64);

    // Extract typed links for the living link system
    let typed_links = extract_typed_links(&content);
    let now = chrono::Utc::now().to_rfc3339();

    // MIG-021 §1A — extract `sources:` from frontmatter (handles all three
    // YAML shapes; unknown values silently dropped). Stored as JSON list
    // in `note_meta.sources` so the Sight v5 mode-P render and the
    // Source Review panel can read in O(1) without re-parsing the file.
    let sources_list = crate::sources::extract_sources(&content);
    let sources_json: Option<String> = if sources_list.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&sources_list).unwrap_or_default())
    };

    // MIG-021v2 §1A' — extract `content_type:` from frontmatter (parallel
    // to `sources:`; same three YAML shapes; validates against the
    // ~218-node vertical taxonomy; unknown values silently dropped).
    let content_type_list = crate::sources::extract_content_type(&content);
    let content_type_json: Option<String> = if content_type_list.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&content_type_list).unwrap_or_default())
    };

    // Upsert: delete old, insert new (triggers handle FTS sync) — wrapped in transaction for atomicity
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        conn.execute("DELETE FROM note_meta WHERE path = ?1", params![note_path])
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, properties_json, tags_json, outgoing_links_json, headings_json, body_text, word_count, created_at, cid_cn, sources, content_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![note_path, name, library_name, modified, props_json, tags_json, links_json, headings_json, plain_body, word_count, created_at, cid_cn, sources_json, content_type_json],
        ).map_err(|e| format!("Failed to index note {}: {}", note_path, e))?;

        // MIG-004 §2: clear+repopulate frontmatter-sourced aliases for
        // this path. The DELETE is partitioned by `source` so any
        // 'rename'-stamped or 'import'-stamped aliases stay put — they
        // have a different lifecycle than the user's `aliases:` list.
        conn.execute(
            "DELETE FROM note_aliases WHERE path = ?1 AND source = 'frontmatter'",
            params![note_path],
        ).map_err(|e| format!("Failed to clear frontmatter aliases: {}", e))?;
        let aliases = extract_aliases(&content);
        if !aliases.is_empty() {
            let mut ins = conn.prepare(
                "INSERT OR IGNORE INTO note_aliases (path, alias_lower, source, cid_cn) VALUES (?1, ?2, 'frontmatter', ?3)"
            ).map_err(|e| format!("prepare alias insert: {}", e))?;
            for a in &aliases {
                ins.execute(params![note_path, a, cid_cn])
                    .map_err(|e| format!("insert alias: {}", e))?;
            }
        }

        // Populate note_links — preserve existing weight/traversal data on re-index
        // Step 1: Snapshot existing traversal data before deleting
        let mut preserved: std::collections::HashMap<String, (f64, String, i64, String, String)> =
            std::collections::HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT target_name, link_type, weight, last_traversed, traversal_count, confidence, created
                 FROM note_links WHERE source_path = ?1"
            ).map_err(|e| e.to_string())?;
            let rows = stmt.query_map(params![note_path], |row| {
                Ok((
                    row.get::<_, String>(0)?,  // target_name
                    row.get::<_, String>(1)?,  // link_type
                    row.get::<_, f64>(2)?,     // weight
                    row.get::<_, String>(3)?,  // last_traversed
                    row.get::<_, i64>(4)?,     // traversal_count
                    row.get::<_, String>(5)?,  // confidence
                    row.get::<_, String>(6)?,  // created
                ))
            }).map_err(|e| e.to_string())?;
            for row in rows {
                if let Ok((target, ltype, w, lt, tc, conf, created)) = row {
                    // Only preserve if link was actually traversed (tc > 0)
                    if tc > 0 || w != 1.0 {
                        let key = format!("{}::{}", ltype, target);
                        preserved.insert(key, (w, lt, tc, conf, created));
                    }
                }
            }
        }
        // Step 2: Delete and re-insert, restoring preserved data
        conn.execute("DELETE FROM note_links WHERE source_path = ?1", params![note_path])
            .map_err(|e| e.to_string())?;
        for tl in &typed_links {
            let key = format!("{}::{}", tl.link_type, tl.target);
            // MIG-003 Step 3: target_cid_cn looked up via note_meta.name
            // (case-folded against the wikilink target). NULL when the
            // target is unresolved — caller is responsible for treating
            // unresolved links as orphans, same as before.
            let target_cid_cn: Option<String> = conn.query_row(
                "SELECT cid_cn FROM note_meta WHERE LOWER(name) = LOWER(?1) LIMIT 1",
                params![tl.target],
                |row| row.get(0),
            ).ok();
            if let Some((w, lt, tc, conf, created)) = preserved.get(&key) {
                conn.execute(
                    "INSERT OR IGNORE INTO note_links (source_path, source_name, target_name, link_type, annotation, confidence, weight, created, last_traversed, traversal_count, library_name, status, source_cid_cn, target_cid_cn)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'active', ?12, ?13)",
                    params![note_path, name, tl.target, tl.link_type, tl.annotation, conf, w, created, lt, tc, library_name, cid_cn, target_cid_cn],
                ).map_err(|e| format!("Failed to index link: {}", e))?;
            } else {
                conn.execute(
                    "INSERT OR IGNORE INTO note_links (source_path, source_name, target_name, link_type, annotation, confidence, weight, created, last_traversed, traversal_count, library_name, status, source_cid_cn, target_cid_cn)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'hypothesis', 1.0, ?6, ?6, 0, ?7, 'active', ?8, ?9)",
                    params![note_path, name, tl.target, tl.link_type, tl.annotation, now, library_name, cid_cn, target_cid_cn],
                ).map_err(|e| format!("Failed to index link: {}", e))?;
            }
        }

        Ok(())
    })();
    match result {
        Ok(()) => { conn.execute_batch("COMMIT").map_err(|e| e.to_string())?; }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK"); return Err(e); }
    }

    Ok(())
}

/// Index all notes in a library directory.
fn index_library_recursive(conn: &Connection, dir: &Path, library_name: &str, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match std::fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            index_library_recursive(conn, &path, library_name, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let _ = index_note(conn, &path.to_string_lossy(), library_name);
        }
    }
}

// ─── Search Execution ──────────────────────────────────────────

/// Lexical search using FTS5 BM25 ranking.
fn lexical_search(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    // Normalize query for Arabic consistency (same normalization as indexed text)
    let normalized = normalize_arabic_for_search(query);

    // M12 wire-up: try cross-language expansion via the lexicon. When the
    // query detects as a supported language and the lemma is in the
    // corpus, we get a phrase-quoted OR-joined expression that pulls in
    // translations and synonyms. Otherwise we fall back to the original
    // prefix-match — that preserves today's behavior for proper nouns,
    // code, rare words, and anything outside our ~20K-concept corpus.
    //
    // M13 badge: when the expanded path fires, the returned
    // `LexicalExpansion` also carries the set of non-source-language
    // lemmas that could have caused a hit. For each returned row we
    // scan the FTS5 snippet for a `<mark>…</mark>` whose contents
    // matches one of those bridge terms — that's the lemma the UI
    // renders as "via {lemma}".
    let expansion = expanded_match_query(&normalized);
    let fts_query = match &expansion {
        Some(e) => e.match_expr.clone(),
        None => format!("{}*", normalized.replace('"', "")),
    };
    let bridge_terms: &[String] = expansion
        .as_ref()
        .map(|e| e.bridge_terms_lower.as_slice())
        .unwrap_or(&[]);

    let mut stmt = match conn.prepare(
        "SELECT note_meta.path, note_meta.name, note_meta.library_name, note_meta.modified,
                bm25(notes_fts, 10.0, 1.0) as score,
                snippet(notes_fts, 1, '<mark>', '</mark>', '...', 40) as snip
         FROM notes_fts
         JOIN note_meta ON notes_fts.rowid = note_meta.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY score
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let query_lower = normalized.to_lowercase();

    let results = stmt.query_map(params![fts_query, limit], |row| {
        let name: String = row.get(1)?;
        let name_lower = name.to_lowercase();
        let title_hit = name_lower.contains(&query_lower);
        let snippet: Option<String> = row.get(5).ok();
        let body_hit = snippet.as_ref().map_or(false, |s| s.contains("<mark>"));

        let match_type = if title_hit && body_hit {
            "title".to_string() // prioritize title when both match
        } else if title_hit {
            "title".to_string()
        } else {
            "content".to_string()
        };

        // M13: report "via {lemma}" only when a cross-language lemma
        // was actually highlighted by FTS5. Title hits short-circuit —
        // a filename match is never a translation event.
        let match_via = if title_hit {
            None
        } else {
            snippet
                .as_deref()
                .and_then(|s| find_match_via(s, bridge_terms))
        };

        Ok(SearchResult {
            path: row.get(0)?,
            name,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: row.get::<_, f64>(4)?.abs(),
            snippet,
            match_type,
            heading_breadcrumb: None,
            match_via,
        })
    }).ok();

    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

/// Output of `expanded_match_query` — the FTS5 MATCH expression plus
/// the set of bridge terms used to surface "via {lemma}" badges.
///
/// `bridge_terms_lower` holds only **non-source-language** lemmas
/// (lowercased up-front so per-row snippet scans skip allocation).
/// A query in English expands to en-plus-everything-else — we filter
/// `En` out so "tree" matching a note containing "trees" (same lang,
/// plural inflection) doesn't get badged as a translation. Only true
/// cross-lingual hits show the badge.
pub(crate) struct LexicalExpansion {
    /// FTS5 MATCH clause, e.g. `"tree" OR "trees" OR "شجرة" OR "árbol"`.
    match_expr: String,
    /// Lowercased non-source-language lemmas from the expansion.
    /// Empty when expansion only produced same-language terms
    /// (`lexical_search` still takes the expanded path in that case
    /// but no row can earn a badge).
    bridge_terms_lower: Vec<String>,
}

impl LexicalExpansion {
    /// Consume into `(match_expr, bridge_terms_lower)` for callers that
    /// want both halves and don't care about the wrapper. Avoids cloning
    /// when the expansion is short-lived (the typical case — built by
    /// `expanded_match_query` and immediately decomposed by callers).
    pub(crate) fn into_parts(self) -> (String, Vec<String>) {
        (self.match_expr, self.bridge_terms_lower)
    }
}

/// Try to produce a cross-language FTS5 MATCH expression for `normalized`
/// via the Lexical Bridge (M10 + M11-data), along with the bridge-term
/// set for M13 badging.
///
/// Returns `Some(LexicalExpansion)` only when expansion actually adds
/// terms beyond the source lemma — detected by the presence of `" OR "`
/// in the built expression. Otherwise returns `None` so `lexical_search`
/// falls back to prefix matching.
///
/// That fallback matters: `expand_to_match_expr` happily returns a
/// single-quoted lemma for in-corpus words that happen to have no
/// translations yet, and a one-term exact-phrase match would *regress*
/// recall versus today's `word*` prefix query. Requiring the " OR "
/// bridge means we only take the expanded path when there's an actual
/// cross-lingual or cross-synonym win.
pub(crate) fn expanded_match_query(normalized: &str) -> Option<LexicalExpansion> {
    let source_lang = crate::lexicon::detect_source_lang(normalized)?;
    let result = crate::lexicon::expand(
        normalized,
        source_lang,
        &crate::lexicon::ExpansionOptions::default(),
    );
    let match_expr = crate::lexicon::fts::build_match_expr(&result)?;
    if !match_expr.contains(" OR ") {
        return None;
    }
    // Filter to cross-language terms only. Same-language expansion
    // (plurals, inflections, in-language synonyms) stays in the MATCH
    // clause so FTS5 finds it, but it doesn't earn a "via" badge —
    // the user already knows their own language's word for it.
    let bridge_terms_lower: Vec<String> = result
        .flat_terms()
        .into_iter()
        .filter(|(lang, _)| *lang != source_lang)
        .map(|(_, term)| term.to_lowercase())
        .collect();
    Some(LexicalExpansion {
        match_expr,
        bridge_terms_lower,
    })
}

/// M13: scan an FTS5 snippet for the first `<mark>…</mark>` whose
/// contents (case-folded) matches a bridge term. Returns the bridge
/// term so the UI can render it as "via {lemma}".
///
/// `bridge_terms_lower` MUST be pre-lowercased — the scan does one
/// `to_lowercase()` per marked region (typically 1–3 per snippet) and
/// compares directly. Anchoring on `<mark>` tags (not raw substring)
/// avoids false positives where a bridge term happens to appear in
/// the unmarked context window around the real match.
///
/// Returns `None` when:
///   - The snippet has no `<mark>` tags (no FTS hit, unreachable in
///     practice — we're called only when `body_hit` is true).
///   - None of the marked regions match a bridge term (the hit came
///     via the source lemma itself — same-language match).
///   - The caller passed an empty `bridge_terms_lower` (the expansion
///     didn't produce cross-language terms, so no badge is possible).
fn find_match_via(snippet: &str, bridge_terms_lower: &[String]) -> Option<String> {
    find_match_via_marked(snippet, bridge_terms_lower, "<mark>", "</mark>")
}

/// Generalized form of [`find_match_via`] that accepts the open/close
/// delimiter pair surrounding marked regions. The lexical-search path
/// uses `<mark>…</mark>` (HTML-escaped on the way out); the Index path
/// (`libraries::read_term_mentions`) uses `CHAR(2)…CHAR(3)` (STX/ETX
/// sentinels) to avoid letting user content inject DOM. Same scan
/// semantics either way — only the delimiters differ.
pub(crate) fn find_match_via_marked(
    snippet: &str,
    bridge_terms_lower: &[String],
    mark_open: &str,
    mark_close: &str,
) -> Option<String> {
    if bridge_terms_lower.is_empty() {
        return None;
    }
    let mut cursor = 0;
    while let Some(open_rel) = snippet[cursor..].find(mark_open) {
        let content_start = cursor + open_rel + mark_open.len();
        let tail = &snippet[content_start..];
        let Some(close_rel) = tail.find(mark_close) else {
            break;
        };
        let marked = &tail[..close_rel];
        let marked_lower = marked.to_lowercase();
        for term in bridge_terms_lower {
            if marked_lower == *term {
                return Some(term.clone());
            }
        }
        cursor = content_start + close_rel + mark_close.len();
    }
    None
}

/// Structured filter search (properties, tags, wikilinks).
fn structured_search(conn: &Connection, filters: &SearchFilters, limit: u32) -> Vec<SearchResult> {
    let mut conditions = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    // Property filters
    if let Some(props) = &filters.properties {
        for pf in props {
            match pf.op.as_str() {
                "=" => {
                    conditions.push(format!("json_extract(properties_json, '$.{}') = ?", pf.key));
                    params_vec.push(Box::new(pf.value.clone().unwrap_or_default()));
                }
                "!=" => {
                    conditions.push(format!("(json_extract(properties_json, '$.{}') IS NULL OR json_extract(properties_json, '$.{}') != ?)", pf.key, pf.key));
                    params_vec.push(Box::new(pf.value.clone().unwrap_or_default()));
                }
                "contains" => {
                    conditions.push(format!("json_extract(properties_json, '$.{}') LIKE '%' || ? || '%'", pf.key));
                    params_vec.push(Box::new(pf.value.clone().unwrap_or_default()));
                }
                "is_empty" => {
                    conditions.push(format!("(json_extract(properties_json, '$.{}') IS NULL OR json_extract(properties_json, '$.{}') = '')", pf.key, pf.key));
                }
                _ => {}
            }
        }
    }

    // Tag filters — JSON-quoted match for exact tag element
    if let Some(tags) = &filters.tags {
        for tag in tags {
            conditions.push("tags_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(tag.to_lowercase()));
        }
    }

    // Wikilink-to filters (find notes that link TO target) — JSON-quoted exact match
    if let Some(targets) = &filters.wikilinks_to {
        for target in targets {
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target.to_lowercase()));
        }
    }

    // Wikilink-from filters: find notes that X links TO (outgoing links of X)
    // This is a two-step query: first find X's outgoing links, then return those notes
    let mut from_targets: Vec<String> = Vec::new();
    if let Some(sources) = &filters.wikilinks_from {
        for source in sources {
            let source_lower = source.to_lowercase();
            // Find the note named `source` and read its outgoing_links_json
            // Try exact match first, then partial (LIKE) for user-typed partial names
            let links: Option<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![source_lower],
                |row| row.get(0),
            ).ok().or_else(|| {
                conn.query_row(
                    "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) LIKE '%' || ?1 || '%' LIMIT 1",
                    params![source_lower],
                    |row| row.get(0),
                ).ok()
            });
            if let Some(links_json) = links {
                if let Ok(targets) = serde_json::from_str::<Vec<String>>(&links_json) {
                    from_targets.extend(targets);
                }
            }
        }
    }
    if !from_targets.is_empty() {
        let placeholders: Vec<String> = from_targets.iter().map(|_| "LOWER(name) = ?".to_string()).collect();
        conditions.push(format!("({})", placeholders.join(" OR ")));
        for t in &from_targets {
            params_vec.push(Box::new(t.clone()));
        }
    }

    // Mutual filters: notes that link to X AND X links back to them
    if let Some(targets) = &filters.mutual {
        for target in targets {
            let target_lower = target.to_lowercase();
            // Must link TO target
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target_lower.clone()));
            // AND this note must be in target's outgoing links (X links back)
            let links: Option<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![target_lower],
                |row| row.get(0),
            ).ok().or_else(|| {
                conn.query_row(
                    "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) LIKE '%' || ?1 || '%' LIMIT 1",
                    params![target_lower],
                    |row| row.get(0),
                ).ok()
            });
            let back_targets: Vec<String> = links
                .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                .unwrap_or_default();
            if !back_targets.is_empty() {
                let placeholders: Vec<String> = back_targets.iter().map(|_| "LOWER(name) = ?".to_string()).collect();
                conditions.push(format!("({})", placeholders.join(" OR ")));
                for bt in &back_targets {
                    params_vec.push(Box::new(bt.clone()));
                }
            } else {
                // Target has no outgoing links → mutual is impossible → return nothing
                conditions.push("0 = 1".to_string());
            }
        }
    }

    // Mentions filter: notes that contain X's name in body but do NOT have [[X]] wikilink
    if let Some(names) = &filters.mentions {
        for name in names {
            let name_lower = name.to_lowercase();
            conditions.push("LOWER(body_text) LIKE '%' || ? || '%'".to_string());
            params_vec.push(Box::new(name_lower.clone()));
            conditions.push("outgoing_links_json NOT LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(name_lower.clone()));
            // Exclude the note itself
            conditions.push("LOWER(name) != ?".to_string());
            params_vec.push(Box::new(name_lower));
        }
    }

    // Orphans filter: notes with no incoming or outgoing links
    // Pre-compute incoming link targets in ONE pass (O(n) instead of O(n²))
    if filters.orphans.unwrap_or(false) {
        // No outgoing links
        conditions.push("(outgoing_links_json IS NULL OR outgoing_links_json = '[]')".to_string());

        // Build set of all notes that have incoming links (single scan of outgoing_links_json)
        let mut has_incoming: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Ok(mut stmt) = conn.prepare("SELECT outgoing_links_json FROM note_meta WHERE outgoing_links_json IS NOT NULL AND outgoing_links_json != '[]'") {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                for row in rows.flatten() {
                    if let Ok(targets) = serde_json::from_str::<Vec<String>>(&row) {
                        for t in targets {
                            has_incoming.insert(t);
                        }
                    }
                }
            }
        }

        // Use temp table for efficient SQL NOT IN check
        let _ = conn.execute("CREATE TEMP TABLE IF NOT EXISTS _incoming_targets (name TEXT PRIMARY KEY)", []);
        let _ = conn.execute("DELETE FROM _incoming_targets", []);
        if let Ok(mut ins) = conn.prepare("INSERT OR IGNORE INTO _incoming_targets (name) VALUES (?1)") {
            for name in &has_incoming {
                let _ = ins.execute(params![name]);
            }
        }
        conditions.push("LOWER(name) NOT IN (SELECT name FROM _incoming_targets)".to_string());
    }

    // Links-between filter: notes that link to BOTH X and Y
    if let Some(targets) = &filters.links_between {
        for target in targets {
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target.to_lowercase()));
        }
    }

    // Links-all filter: notes connected to X in either direction (incoming OR outgoing)
    // Results get match_type "wikilink" with snippet indicating direction
    if let Some(targets) = &filters.links_all {
        for target in targets {
            let target_lower = target.to_lowercase();
            // Get X's outgoing links (notes X links to)
            // Exact match first, then partial (LIKE) for user-typed partial names
            let outgoing: Vec<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![target_lower],
                |row| row.get::<_, String>(0),
            ).ok()
            .or_else(|| {
                conn.query_row(
                    "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) LIKE '%' || ?1 || '%' LIMIT 1",
                    params![target_lower],
                    |row| row.get::<_, String>(0),
                ).ok()
            })
            .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
            .unwrap_or_default();

            // Build: (links TO X) OR (X links to this note)
            let mut sub_conditions: Vec<String> = Vec::new();
            // Incoming: notes that link to X
            sub_conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target_lower.clone()));
            // Outgoing: notes that X links to
            for out_name in &outgoing {
                sub_conditions.push("LOWER(name) = ?".to_string());
                params_vec.push(Box::new(out_name.clone()));
            }
            conditions.push(format!("({})", sub_conditions.join(" OR ")));
            // Exclude X itself
            conditions.push("LOWER(name) != ?".to_string());
            params_vec.push(Box::new(target_lower));
        }
    }

    // Typed link filter: find notes that have a specific relationship to a target
    // e.g., "supports [[X]]" → find notes where source has link_type=supports to target=X
    if let Some(typed_links) = &filters.typed_links {
        for tl in typed_links {
            let target_lower = tl.target.to_lowercase();
            let link_type_lower = tl.link_type.to_lowercase();
            // Query note_links table for matching typed links
            let mut source_paths: Vec<String> = Vec::new();
            if let Ok(mut stmt) = conn.prepare(
                "SELECT DISTINCT source_path FROM note_links WHERE link_type = ?1 AND (LOWER(target_name) = ?2 OR LOWER(target_name) LIKE '%' || ?2 || '%') AND status = 'active'"
            ) {
                if let Ok(rows) = stmt.query_map(params![link_type_lower, target_lower], |row| row.get::<_, String>(0)) {
                    for row in rows.flatten() {
                        source_paths.push(row);
                    }
                }
            }
            if source_paths.is_empty() {
                // No matches — add impossible condition to return empty results
                conditions.push("1 = 0".to_string());
            } else {
                let placeholders: Vec<String> = source_paths.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("path IN ({})", placeholders.join(",")));
                for sp in &source_paths {
                    params_vec.push(Box::new(sp.clone()));
                }
            }
        }
    }

    // Library filter
    if let Some(libs) = &filters.library_names {
        if !libs.is_empty() {
            let placeholders: Vec<String> = libs.iter().enumerate().map(|(_, _)| "?".to_string()).collect();
            conditions.push(format!("library_name IN ({})", placeholders.join(",")));
            for lib in libs {
                params_vec.push(Box::new(lib.clone()));
            }
        }
    }

    // Path prefix filter
    if let Some(prefix) = &filters.path_prefix {
        conditions.push("path LIKE ? || '%'".to_string());
        params_vec.push(Box::new(prefix.clone()));
    }

    if conditions.is_empty() {
        return Vec::new();
    }

    // Determine the dominant filter type for match_type coloring
    let dominant_type = if filters.tags.as_ref().map_or(false, |t| !t.is_empty()) {
        "tag"
    } else if filters.wikilinks_to.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_to"
    } else if filters.wikilinks_from.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_from"
    } else if filters.mutual.as_ref().map_or(false, |w| !w.is_empty()) {
        "mutual"
    } else if filters.links_between.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_between"
    } else if filters.links_all.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_all"
    } else if filters.mentions.as_ref().map_or(false, |m| !m.is_empty()) {
        "mentions"
    } else if filters.orphans.unwrap_or(false) {
        "orphan"
    } else if filters.properties.as_ref().map_or(false, |p| !p.is_empty()) {
        "property"
    } else {
        "structured"
    };

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT path, name, library_name, modified FROM note_meta WHERE {} ORDER BY modified DESC LIMIT {}",
        where_clause, limit
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => { eprintln!("[Search] SQL error: {}", e); return Vec::new(); }
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let mt = dominant_type.to_string();

    let results = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            match_type: mt.clone(),
            snippet: None,
            heading_breadcrumb: None,
            match_via: None,
        })
    }).ok();

    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

// ─── Link System Queries ──────────────────────────────────────

/// Get link statistics from the note_links table.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkStats {
    pub total_links: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    pub by_confidence: std::collections::HashMap<String, usize>,
    pub with_annotation: usize,
    pub sample_links: Vec<serde_json::Value>,
}

#[tauri::command]
pub fn constellation_link_stats(app: tauri::AppHandle) -> Result<LinkStats, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let total_links: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active'", [], |r| r.get(0)
    ).unwrap_or(0);

    let mut by_type = std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT link_type, COUNT(*) FROM note_links WHERE status = 'active' GROUP BY link_type") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,usize>(1)?))) {
            for row in rows.flatten() { by_type.insert(row.0, row.1); }
        }
    }

    let mut by_confidence = std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT confidence, COUNT(*) FROM note_links WHERE status = 'active' GROUP BY confidence") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,usize>(1)?))) {
            for row in rows.flatten() { by_confidence.insert(row.0, row.1); }
        }
    }

    let with_annotation: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active' AND annotation != ''", [], |r| r.get(0)
    ).unwrap_or(0);

    let mut sample_links = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT source_name, target_name, link_type, annotation, confidence, weight FROM note_links WHERE status = 'active' ORDER BY weight DESC LIMIT 10"
    ) {
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok(serde_json::json!({
                "source": r.get::<_,String>(0)?,
                "target": r.get::<_,String>(1)?,
                "type": r.get::<_,String>(2)?,
                "annotation": r.get::<_,String>(3)?,
                "confidence": r.get::<_,String>(4)?,
                "weight": r.get::<_,f64>(5)?,
            }))
        }) {
            for row in rows.flatten() { sample_links.push(row); }
        }
    }

    Ok(LinkStats { total_links, by_type, by_confidence, with_annotation, sample_links })
}

/// Record a link traversal: user followed a link from source to target.
/// Updates last_traversed, increments traversal_count, recalculates weight.
/// Weight formula: 1.0 + ln(1 + traversal_count) — logarithmic, early traversals matter most.
#[tauri::command]
pub fn constellation_link_traverse(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
) -> Result<serde_json::Value, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let now = chrono::Utc::now().to_rfc3339();
    let target_lower = target_name.to_lowercase();

    // Two-step: read current traversal_count, compute new weight in Rust, then update.
    // This avoids reliance on SQLite math functions (ln) which need SQLITE_ENABLE_MATH_FUNCTIONS.
    let mut stmt = conn.prepare(
        "SELECT id, traversal_count FROM note_links
         WHERE source_path = ?1 AND LOWER(target_name) = ?2"
    ).map_err(|e| e.to_string())?;
    let links: Vec<(i64, i64)> = stmt.query_map(params![source_path, target_lower], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| e.to_string())?
      .filter_map(|r| r.ok())
      .collect();

    let mut updated: usize = 0;
    for (id, tc) in &links {
        let new_tc = tc + 1;
        let new_weight = 1.0 + (1.0 + new_tc as f64).ln();
        // P5 slice 3: confidence auto-promotion on traversal.
        // Tiers align with the frontend `LinkLifecycle` thresholds:
        //   3+ traversals  → "evidence"     (matches UI tier "established")
        //   10+ traversals → "established"  (matches UI tier "load-bearing")
        // A "contested" state remains reserved for user-driven promotion
        // via a future write path — we never auto-downgrade in this pass.
        // CASE WHEN preserves any user-promoted value that outranks the
        // auto-tier ("contested" / "established" when the count is still
        // climbing).
        let new_confidence = if new_tc >= 10 {
            "established"
        } else if new_tc >= 3 {
            "evidence"
        } else {
            "hypothesis"
        };
        conn.execute(
            "UPDATE note_links SET
                traversal_count = ?1,
                last_traversed = ?2,
                weight = ?3,
                status = CASE WHEN status = 'dormant' THEN 'active' ELSE status END,
                confidence = CASE
                    WHEN confidence = 'contested' THEN confidence
                    WHEN confidence = 'established' THEN confidence
                    WHEN confidence = 'evidence' AND ?5 = 'hypothesis' THEN confidence
                    ELSE ?5
                END
             WHERE id = ?4",
            params![new_tc, now, new_weight, id, new_confidence],
        ).map_err(|e| format!("Failed to record traversal: {}", e))?;
        updated += 1;
    }

    Ok(serde_json::json!({
        "updated": updated,
        "source": source_path,
        "target": target_name,
        "timestamp": now,
    }))
}

/// Developer read-back for a single (source_path, target_name) pair. Returns
/// every matching row's lifecycle fields raw. Used to validate throttle and
/// traversal behavior before the P3 visual surfaces exist. No UI depends on
/// this; safe to remove once Backlinks/Outgoing panels render the data.
#[tauri::command]
pub fn constellation_debug_link_state(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target_name.to_lowercase();
    let mut stmt = conn.prepare(
        "SELECT id, source_name, target_name, link_type, confidence,
                traversal_count, weight, last_traversed, status, annotation
         FROM note_links
         WHERE source_path = ?1 AND LOWER(target_name) = ?2"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![source_path, target_lower], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "source_name": row.get::<_, String>(1)?,
            "target_name": row.get::<_, String>(2)?,
            "link_type": row.get::<_, String>(3)?,
            "confidence": row.get::<_, String>(4)?,
            "traversal_count": row.get::<_, i64>(5)?,
            "weight": row.get::<_, f64>(6)?,
            "last_traversed": row.get::<_, String>(7).unwrap_or_default(),
            "status": row.get::<_, String>(8)?,
            "annotation": row.get::<_, String>(9).unwrap_or_default(),
        }))
    }).map_err(|e| e.to_string())?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Find dormant links — links not traversed within the given threshold (default 90 days).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DormantLink {
    pub source_name: String,
    pub target_name: String,
    pub link_type: String,
    pub annotation: String,
    pub weight: f64,
    pub last_traversed: String,
    pub traversal_count: i64,
    pub days_dormant: i64,
}

#[tauri::command]
pub fn constellation_link_dormant(
    app: tauri::AppHandle,
    days_threshold: Option<u32>,
) -> Result<Vec<DormantLink>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let threshold = days_threshold.unwrap_or(90) as i64;

    let mut stmt = conn.prepare(
        "SELECT source_name, target_name, link_type, annotation, weight,
                last_traversed, traversal_count,
                CAST(julianday('now') - julianday(last_traversed) AS INTEGER) AS days_dormant
         FROM note_links
         WHERE status = 'active'
           AND last_traversed != ''
           AND julianday('now') - julianday(last_traversed) >= ?1
         ORDER BY days_dormant DESC
         LIMIT 200"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![threshold], |row| {
        Ok(DormantLink {
            source_name: row.get(0)?,
            target_name: row.get(1)?,
            link_type: row.get(2)?,
            annotation: row.get(3)?,
            weight: row.get(4)?,
            last_traversed: row.get(5)?,
            traversal_count: row.get(6)?,
            days_dormant: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ─── P4: Formulation Analysis (Knowledge Diagnostics) ────────

/// A formulation insight — one row from a diagnostic query.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FormulationInsight {
    pub source_name: String,
    pub target_name: String,
    pub link_type: String,
    pub annotation: String,
    pub weight: f64,
    pub confidence: String,
    pub traversal_count: i64,
    pub last_traversed: String,
    pub library_name: String,
}

/// Formulation analysis: diagnostic queries for intellectual life.
/// `query_type` determines which analysis runs:
///   - "strongest_evidence"  — top supports for a target, ranked by weight × confidence
///   - "weak_foundations"    — hypothesis links with high weight (building on sand)
///   - "tensions"           — contradicts links for a target
///   - "stagnating"         — high-weight links gone dormant
///   - "abandoned"          — archived links
///   - "emerging"           — hypothesis + growing weight (curiosity without proof)
///   - "bias_check"         — targets where supports >> contradicts (echo chambers)
///   - "most_connected"     — notes with most incoming typed links
///   - "knowledge_gaps"     — notes with outgoing links but few incoming (giving but not receiving)
#[tauri::command]
pub fn constellation_formulation_analysis(
    app: tauri::AppHandle,
    query_type: String,
    target: Option<String>,
) -> Result<Vec<FormulationInsight>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target.as_deref().unwrap_or("").to_lowercase();
    let confidence_weight = |c: &str| -> f64 {
        match c { "established" => 3.0, "evidence" => 2.0, "hypothesis" => 1.0, "contested" => 0.5, _ => 1.0 }
    };

    match query_type.as_str() {
        "strongest_evidence" => {
            // Top supports for a target, ranked by weight × confidence multiplier
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE link_type = 'supports' AND status = 'active'
                 AND (?1 = '' OR LOWER(target_name) LIKE '%' || ?1 || '%')
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            let mut results = query_insights(&mut stmt, &[&target_lower as &dyn rusqlite::types::ToSql])?;
            // Re-sort by weight × confidence
            results.sort_by(|a, b| {
                let sa = a.weight * confidence_weight(&a.confidence);
                let sb = b.weight * confidence_weight(&b.confidence);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            Ok(results)
        }
        "weak_foundations" => {
            // hypothesis links with high weight — building on uncertain ground
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE confidence = 'hypothesis' AND weight > 2.0 AND status = 'active'
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "tensions" => {
            // contradicts links for a target
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE link_type = 'contradicts' AND status = 'active'
                 AND (?1 = '' OR LOWER(target_name) LIKE '%' || ?1 || '%')
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[&target_lower as &dyn rusqlite::types::ToSql])
        }
        "stagnating" => {
            // high-weight links gone dormant
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE status = 'dormant' AND weight > 2.0
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "abandoned" => {
            // archived links
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE status = 'archived'
                 ORDER BY last_traversed DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "emerging" => {
            // hypothesis + recently traversed (curiosity without proof yet)
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE confidence = 'hypothesis' AND traversal_count > 0 AND status = 'active'
                 ORDER BY traversal_count DESC, weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "bias_check" => {
            // targets where supports count >> contradicts count
            let mut stmt = conn.prepare(
                "SELECT target_name,
                    SUM(CASE WHEN link_type = 'supports' THEN 1 ELSE 0 END) as support_count,
                    SUM(CASE WHEN link_type = 'contradicts' THEN 1 ELSE 0 END) as contradict_count
                 FROM note_links WHERE status = 'active' AND link_type IN ('supports', 'contradicts')
                 GROUP BY target_name
                 HAVING support_count > 0 AND contradict_count = 0
                 ORDER BY support_count DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |row| {
                Ok(FormulationInsight {
                    source_name: String::new(),
                    target_name: row.get(0)?,
                    link_type: "bias".to_string(),
                    annotation: format!("{} supports, {} contradicts", row.get::<_, i64>(1)?, row.get::<_, i64>(2)?),
                    weight: row.get::<_, i64>(1)? as f64,
                    confidence: String::new(),
                    traversal_count: 0,
                    last_traversed: String::new(),
                    library_name: String::new(),
                })
            }).map_err(|e| e.to_string())?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }
        "most_connected" => {
            // notes with most incoming typed links
            let mut stmt = conn.prepare(
                "SELECT target_name, COUNT(*) as cnt, GROUP_CONCAT(DISTINCT link_type) as types,
                        AVG(weight) as avg_weight
                 FROM note_links WHERE status = 'active'
                 GROUP BY target_name ORDER BY cnt DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |row| {
                Ok(FormulationInsight {
                    source_name: String::new(),
                    target_name: row.get(0)?,
                    link_type: row.get::<_, String>(2).unwrap_or_default(),
                    annotation: format!("{} incoming links", row.get::<_, i64>(1)?),
                    weight: row.get::<_, f64>(3).unwrap_or(1.0),
                    confidence: String::new(),
                    traversal_count: row.get::<_, i64>(1)?,
                    last_traversed: String::new(),
                    library_name: String::new(),
                })
            }).map_err(|e| e.to_string())?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }
        _ => Err(format!("Unknown formulation query: {}", query_type)),
    }
}

/// Helper: execute a prepared statement and collect FormulationInsight rows.
fn query_insights(
    stmt: &mut rusqlite::Statement,
    params: &[&dyn rusqlite::types::ToSql],
) -> Result<Vec<FormulationInsight>, String> {
    let rows = stmt.query_map(params, |row| {
        Ok(FormulationInsight {
            source_name: row.get(0)?,
            target_name: row.get(1)?,
            link_type: row.get(2)?,
            annotation: row.get(3)?,
            weight: row.get(4)?,
            confidence: row.get(5)?,
            traversal_count: row.get(6)?,
            last_traversed: row.get(7)?,
            library_name: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Apply weight decay to all active links.
/// Formula: weight = weight × 0.95^(months_since_last_traversal)
/// Only decays links not traversed in the last 30 days.
/// Also derives lifecycle stage from weight + traversal data.
#[tauri::command]
pub fn constellation_link_decay(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    // Step 1: Apply weight decay to links not traversed in 30+ days
    // Calculate months since last traversal for each link
    let mut decayed: usize = 0;
    let mut dormant_count: usize = 0;

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, weight, last_traversed, traversal_count, status
         FROM note_links
         WHERE status IN ('active', 'dormant')
           AND last_traversed != ''
           AND julianday('now') - julianday(last_traversed) >= 30"
    ) {
        let links: Vec<(i64, f64, String, i64, String)> = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
            ))
        }).map_err(|e| e.to_string())?
          .filter_map(|r| r.ok())
          .collect();

        for (id, weight, last_traversed, _tc, status) in &links {
            // Calculate months since last traversal
            let months = conn.query_row(
                "SELECT (julianday('now') - julianday(?1)) / 30.0",
                params![last_traversed],
                |row| row.get::<_, f64>(0),
            ).unwrap_or(0.0);

            if months < 1.0 { continue; }

            // Apply decay: weight × 0.95^months
            let new_weight = weight * (0.95_f64).powf(months);
            let new_weight = (new_weight * 1000.0).round() / 1000.0; // round to 3 decimals

            // Determine lifecycle status
            let new_status = if months >= 3.0 && *status == "active" {
                dormant_count += 1;
                "dormant"
            } else {
                status.as_str()
            };

            conn.execute(
                "UPDATE note_links SET weight = ?1, status = ?2 WHERE id = ?3",
                params![new_weight, new_status, id],
            ).map_err(|e| format!("Failed to decay link: {}", e))?;
            decayed += 1;
        }
    }

    // Step 2: Count lifecycle stage distribution.
    //
    // MIG-014 §2F — buckets aligned with the Living Link 6-stage taxonomy
    // (`LIVING_LINK_BASELINE` in `src/lib/libraries/store.ts`):
    //   spark    — traversal_count = 0 AND created within last 7 days
    //   birth    — traversal_count = 0 AND created ≥ 7 days ago
    //   growth   — traversal_count > 0 AND weight < 5.0
    //   maturity — weight >= 5.0
    //   dormancy — status = 'dormant'
    //   archival — status = 'archived' (DB enum stays 'archived'; dashboard
    //              key is `archival` to match the lifecycle name)
    let mut stages: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Spark: just-created and untraversed (< 7 days). SQLite's julianday
    // does the date math without needing client-side parsing.
    let spark: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links \
         WHERE status = 'active' AND traversal_count = 0 \
           AND julianday('now') - julianday(created) < 7",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("spark".to_string(), spark);

    // Birth: still untraversed but past the spark window.
    let birth: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links \
         WHERE status = 'active' AND traversal_count = 0 \
           AND julianday('now') - julianday(created) >= 7",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("birth".to_string(), birth);

    // Growth: traversed at least once, not yet mature.
    let growth: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active' AND traversal_count > 0 AND weight < 5.0",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("growth".to_string(), growth);

    // Maturity.
    let maturity: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active' AND weight >= 5.0",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("maturity".to_string(), maturity);

    // Dormancy.
    let dormant: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'dormant'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("dormancy".to_string(), dormant);

    // Archival (DB enum stays 'archived' — back-compat). Bucket key uses
    // `archival` to match the lifecycle name and the frontend `stageColors`.
    let archival: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'archived'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("archival".to_string(), archival);

    Ok(serde_json::json!({
        "decayed": decayed,
        "new_dormant": dormant_count,
        "lifecycle": stages,
    }))
}

/// Update a link's confidence level.
#[tauri::command]
pub fn constellation_link_set_confidence(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
    confidence: String,
) -> Result<(), String> {
    if !["hypothesis", "evidence", "established", "contested"].contains(&confidence.as_str()) {
        return Err(format!("Invalid confidence level: {}", confidence));
    }
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target_name.to_lowercase();
    conn.execute(
        "UPDATE note_links SET confidence = ?1 WHERE source_path = ?2 AND LOWER(target_name) = ?3",
        params![confidence, source_path, target_lower],
    ).map_err(|e| format!("Failed to update confidence: {}", e))?;

    Ok(())
}

/// One-shot backfill: age-assign `confidence` for rows that already have
/// enough traversals but were never promoted (e.g. they existed before the
/// auto-promotion rule shipped, or sat at `hypothesis` because every click
/// happened pre-P5-slice-3). Never downgrades; preserves user-set `contested`.
/// Returns counts per tier so the UI can report how many rows moved.
#[tauri::command]
pub fn constellation_link_backfill_confidence(
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let to_established: usize = conn.execute(
        "UPDATE note_links
         SET confidence = 'established'
         WHERE confidence NOT IN ('established', 'contested')
           AND traversal_count >= 10",
        [],
    ).map_err(|e| format!("Failed to backfill established: {}", e))?;

    let to_evidence: usize = conn.execute(
        "UPDATE note_links
         SET confidence = 'evidence'
         WHERE confidence = 'hypothesis'
           AND traversal_count >= 3
           AND traversal_count < 10",
        [],
    ).map_err(|e| format!("Failed to backfill evidence: {}", e))?;

    Ok(serde_json::json!({
        "promoted_to_established": to_established,
        "promoted_to_evidence": to_evidence,
        "total": to_established + to_evidence,
    }))
}

/// Archive a link (soft delete — preserved in history).
#[tauri::command]
pub fn constellation_link_archive(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target_name.to_lowercase();
    conn.execute(
        "UPDATE note_links SET status = 'archived', weight = 0.0 WHERE source_path = ?1 AND LOWER(target_name) = ?2",
        params![source_path, target_lower],
    ).map_err(|e| format!("Failed to archive link: {}", e))?;

    Ok(())
}

/// Resurrect an archived link. Resets weight to 1.0 (baseline) and status
/// back to 'active'. Traversal count and confidence are preserved so the
/// link's history isn't lost.
#[tauri::command]
pub fn constellation_link_unarchive(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target_name.to_lowercase();
    conn.execute(
        "UPDATE note_links SET status = 'active', weight = 1.0 WHERE source_path = ?1 AND LOWER(target_name) = ?2",
        params![source_path, target_lower],
    ).map_err(|e| format!("Failed to unarchive link: {}", e))?;

    Ok(())
}

/// List archived links for the Link Dashboard's Archived tab.
#[tauri::command]
pub fn constellation_link_archived(app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let mut stmt = conn.prepare(
        "SELECT source_path, source_name, target_name, link_type, annotation, confidence,
                traversal_count, last_traversed, library_name
         FROM note_links
         WHERE status = 'archived'
         ORDER BY last_traversed DESC, source_name ASC",
    ).map_err(|e| format!("Failed to prepare archived-links query: {}", e))?;

    let rows: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "source_path":     row.get::<_, String>(0)?,
            "source_name":     row.get::<_, String>(1)?,
            "target_name":     row.get::<_, String>(2)?,
            "link_type":       row.get::<_, String>(3).unwrap_or_default(),
            "annotation":      row.get::<_, String>(4).unwrap_or_default(),
            "confidence":      row.get::<_, String>(5).unwrap_or_default(),
            "traversal_count": row.get::<_, i64>(6).unwrap_or(0),
            "last_traversed":  row.get::<_, String>(7).unwrap_or_default(),
            "library_name":    row.get::<_, String>(8).unwrap_or_default(),
        }))
    }).map_err(|e| e.to_string())?
      .filter_map(|r| r.ok())
      .collect();

    Ok(rows)
}

// ─── Tauri Commands ────────────────────────────────────────────

/// Fast path: open the search DB (creating schema if absent) and place it in
/// state. Does NOT walk the filesystem. Safe to call from the boot path — on
/// a populated DB this is a millisecond-scale operation.
///
/// Previously `constellation_search_init` opened the DB AND walked every
/// library before putting the connection in state. That meant any concurrent
/// `cache_boot_snapshot` call saw `None` and reported a cold cache, defeating
/// the whole cache-first boot. Splitting this in two is what makes the
/// cache-first boot actually work on 2nd+ launches.
pub fn ensure_search_db_ready(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<SearchState>();
    {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Ok(());
        }
    }
    let path = db_path(app)?;
    let version_path = path.with_extension("version");
    let current_version = "7";
    let needs_rebuild = match std::fs::read_to_string(&version_path) {
        Ok(v) => v.trim() != current_version,
        Err(_) => true,
    };
    if needs_rebuild {
        let _ = std::fs::remove_file(&path);
    }
    let conn = init_db(&path)?;
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        *db = Some(conn);
    }
    if needs_rebuild {
        let _ = std::fs::write(&version_path, current_version);
    }
    // MIG-001 Step 5: schedule the Sky View back-fill on a background
    // thread. No-op if schema_versions.sky is already at target. Returns
    // immediately so this doesn't extend boot time.
    crate::sky_backfill::maybe_schedule(app.clone());

    // MIG-015 (PJ-001) §1B: schedule the deferred term_vocab v2 sentinel
    // migration on a background thread. No-op if the schema stamp is
    // already at v2 OR if there are no rows pending. On a pre-MIG-013
    // library (~5.7M rows) this avoids the 30-90 sec boot freeze that
    // the inline single-statement UPDATE used to cause; the user gets
    // a fully-painted UI immediately and the chunked migration runs in
    // the background with status-bar progress emit.
    maybe_schedule_v2_sentinel_migration(app.clone());

    Ok(())
}

/// Walk every library and reindex changed files using a DEDICATED connection
/// (not the one in SearchState). SQLite's WAL mode allows concurrent readers,
/// so frontend queries through `state.db` continue working while this runs.
///
/// Runs in the caller's thread. `cache_reconcile` wraps this in
/// `std::thread::spawn` so it never blocks IPC.
pub fn reconcile_filesystem(app: &tauri::AppHandle) -> Result<SearchIndexStats, String> {
    // Make sure schema exists and state has the query connection.
    ensure_search_db_ready(app)?;

    let path = db_path(app)?;
    // Dedicated connection for the walk — does NOT touch state.db, so the
    // state's query connection stays available to frontend reads the whole
    // time. WAL mode (set in init_db) is what makes this safe.
    let mut walk_conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open search.db for reconcile: {}", e))?;
    walk_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA recursive_triggers=ON;")
        .map_err(|e| e.to_string())?;
    // Reconcile writes to note_meta; the FTS5 AFTER-INSERT/UPDATE
    // triggers tokenize body_text through the 'constellation' tokenizer.
    // Without registration here the trigger's INSERT INTO notes_fts
    // would fail on this connection with "no such tokenizer".
    register_fts5_tokenizer(&mut walk_conn)?;

    let libraries = crate::libraries::load_all_libraries(app);
    for lib in &libraries {
        index_library_recursive(&walk_conn, Path::new(&lib.path), &lib.name, 0);
    }

    let note_count: u32 = walk_conn.query_row(
        "SELECT COUNT(*) FROM note_meta", [], |row| row.get(0)
    ).unwrap_or(0);
    let index_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    Ok(SearchIndexStats { note_count, index_size_bytes: index_size })
}

/// Initialize the search index — builds/rebuilds the SQLite database.
///
/// Kept for backward compatibility with callers that want the legacy
/// "open + walk" behavior. The boot path now uses `ensure_search_db_ready`
/// (instant) and `reconcile_filesystem` (on a background thread) separately.
#[tauri::command]
pub fn constellation_search_init(app: tauri::AppHandle) -> Result<SearchIndexStats, String> {
    ensure_search_db_ready(&app)?;
    reconcile_filesystem(&app)
}

/// Reindex a single note (called on file change).
#[tauri::command]
pub fn constellation_search_reindex(
    app: tauri::AppHandle,
    note_path: String,
    library_name: String,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    reindex_single_note(&state, &note_path, &library_name)
}

/// Delete a note from the search index + link table.
pub fn reindex_delete_note(state: &SearchState, note_path: &str) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db.as_ref() {
        // MIG-013 §1C — capture body BEFORE deletion so the CTSE hook
        // can subtract this note's term contributions from term_vocab.
        // `note_meta.body_text` is the source of truth for tokenization
        // (matches what `notes_fts` ingested at index time).
        let old_body: Option<String> = conn
            .query_row(
                "SELECT body_text FROM note_meta WHERE path = ?1",
                params![note_path],
                |row| row.get(0),
            )
            .ok();
        let _ = conn.execute("DELETE FROM note_links WHERE source_path = ?1", params![note_path]);
        let _ = conn.execute("DELETE FROM note_meta WHERE path = ?1", params![note_path]);
        if let Some(body) = old_body {
            // Best-effort: term_vocab maintenance failure must not fail
            // the file-level deletion. The file is gone; correctness is
            // recoverable on next save of any other note touching the
            // same terms.
            if let Err(e) = crate::ctse::hooks::on_note_deleted(conn, note_path, &body) {
                eprintln!("[ctse] on_note_deleted failed for {}: {}", note_path, e);
            }
        }
    }
    Ok(())
}

/// Reindex a single note — callable from other modules without Tauri command overhead.
pub fn reindex_single_note(
    state: &SearchState,
    note_path: &str,
    library_name: &str,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db.as_ref() {
        // MIG-013 §1C — capture old body BEFORE index_note overwrites
        // the row. The CTSE hook needs both old and new bodies to
        // compute a signed term-count delta against term_vocab.
        // First-time saves: query returns None, hook treats every new
        // token as fresh contribution.
        let old_body: Option<String> = conn
            .query_row(
                "SELECT body_text FROM note_meta WHERE path = ?1",
                params![note_path],
                |row| row.get(0),
            )
            .ok();

        index_note(conn, note_path, library_name)?;

        // Post-COMMIT (index_note's BEGIN IMMEDIATE/COMMIT block has
        // already returned). Read the freshly-written body and apply
        // the delta. If the file no longer exists on disk index_note
        // becomes a no-op — note_meta.body_text reflects the prior
        // state, but the post-read returns the same as the pre-read,
        // yielding a zero delta. Correct.
        let new_body_opt: Option<String> = conn
            .query_row(
                "SELECT body_text FROM note_meta WHERE path = ?1",
                params![note_path],
                |row| row.get(0),
            )
            .ok();
        if let Some(new_body) = new_body_opt {
            // Best-effort: term_vocab maintenance failure must not
            // fail the file reindex. note_meta + notes_fts are the
            // user-facing sources of truth; term_vocab is a derived
            // view and can be re-synthesized from the corpus.
            if let Err(e) = crate::ctse::hooks::on_note_indexed(
                conn,
                note_path,
                old_body.as_deref(),
                &new_body,
            ) {
                eprintln!("[ctse] on_note_indexed failed for {}: {}", note_path, e);
            }
        }
    }
    Ok(())
}

/// Main search command — supports lexical, structured, and combined modes.
#[tauri::command]
pub fn constellation_search(
    app: tauri::AppHandle,
    request: SearchRequest,
) -> Result<Vec<SearchResult>, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;

    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => {
            // Fallback: open the DB lazily. This used to call
            // constellation_search_init which WALKED every library on every
            // first-search — blocking the UI for seconds on a large Universe.
            // Now we only open the connection (cheap); the index is kept fresh
            // by the file watcher and by explicit Rebuild Index actions. If
            // the index is cold (empty), the search just returns no results.
            drop(db_guard);
            ensure_search_db_ready(&app)?;
            let state = app.state::<SearchState>();
            let db_guard = state.db.lock().map_err(|e| e.to_string())?;
            return match db_guard.as_ref() {
                Some(c) => execute_search(c, &request),
                None => Err("Search index not available".to_string()),
            };
        }
    };

    execute_search(conn, &request)
}

fn execute_search(conn: &Connection, request: &SearchRequest) -> Result<Vec<SearchResult>, String> {
    let limit = if request.limit.unwrap_or(0) == 0 { 100000 } else { request.limit.unwrap() };
    let mut results = Vec::new();

    match request.mode.as_str() {
        "lexical" => {
            if let Some(q) = &request.query {
                if !q.trim().is_empty() {
                    results = lexical_search(conn, q, limit);
                }
            }
        }
        "structured" => {
            if let Some(filters) = &request.filters {
                results = structured_search(conn, filters, limit);
                // Post-process links_all: tag each result with direction (↑ incoming / ↓ outgoing)
                if let Some(targets) = &filters.links_all {
                    for target in targets {
                        let target_lower = target.to_lowercase();
                        let outgoing: Vec<String> = conn.query_row(
                            "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                            params![target_lower],
                            |row| row.get::<_, String>(0),
                        ).ok()
                        .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                        .unwrap_or_default();
                        let outgoing_set: std::collections::HashSet<String> = outgoing.into_iter().collect();

                        for r in results.iter_mut() {
                            let r_lower = r.name.to_lowercase();
                            let is_outgoing = outgoing_set.contains(&r_lower);
                            // Check if this result links TO target (incoming to target)
                            let is_incoming = r.snippet.as_ref()
                                .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                                .map(|links| links.contains(&target_lower))
                                .unwrap_or(false);
                            // If we can't tell from snippet, check outgoing_links_json
                            let is_incoming = is_incoming || {
                                conn.query_row(
                                    "SELECT outgoing_links_json FROM note_meta WHERE path = ?1",
                                    params![r.path],
                                    |row| row.get::<_, String>(0),
                                ).ok()
                                .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                                .map(|links| links.contains(&target_lower))
                                .unwrap_or(false)
                            };

                            r.snippet = Some(if is_incoming && is_outgoing {
                                "↑↓".to_string()
                            } else if is_incoming {
                                "↑".to_string()
                            } else {
                                "↓".to_string()
                            });
                        }
                    }
                }
            }
        }
        "semantic" => {
            // Semantic-only search using stored embeddings
            if let Some(q_embedding) = request.query_embedding.as_ref() {
                results = semantic_search(conn, q_embedding, limit);
            }
        }
        "hybrid" | _ => {
            // Full hybrid: RRF fusion of lexical + semantic + structured
            let mut lexical_results = Vec::new();
            let mut semantic_results = Vec::new();
            let mut structured_results = Vec::new();

            if let Some(q) = &request.query {
                if !q.trim().is_empty() {
                    lexical_results = lexical_search(conn, q, limit * 2);
                }
            }

            if let Some(q_embedding) = request.query_embedding.as_ref() {
                semantic_results = semantic_search(conn, q_embedding, limit * 2);
            }

            if let Some(filters) = &request.filters {
                structured_results = structured_search(conn, filters, limit);
            }

            // RRF fusion: score(d) = Σ 1/(k + rank_i(d)), k=60
            if !lexical_results.is_empty() || !semantic_results.is_empty() {
                results = rrf_fuse(lexical_results, semantic_results, 60);
            }

            // Merge structured results (they're filter-based, not ranked)
            let seen: std::collections::HashSet<String> = results.iter().map(|r| r.path.clone()).collect();
            for r in structured_results {
                if !seen.contains(&r.path) { results.push(r); }
            }
        }
    }

    results.truncate(limit as usize);
    Ok(results)
}

// ─── Semantic Search ───────────────────────────────────────────

/// Cosine similarity between two float vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

/// Search for notes similar to a query embedding using cosine similarity.
fn semantic_search(conn: &Connection, query_embedding: &[f32], limit: u32) -> Vec<SearchResult> {
    // Load all embeddings and compute similarity
    let mut stmt = match conn.prepare(
        "SELECT e.path, m.name, m.library_name, m.modified, e.embedding, e.dimensions
         FROM note_embeddings e
         JOIN note_meta m ON e.path = m.path"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut scored: Vec<(SearchResult, f32)> = Vec::new();

    let rows = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let name: String = row.get(1)?;
        let library_name: String = row.get(2)?;
        let modified: u64 = row.get(3)?;
        let embedding_blob: Vec<u8> = row.get(4)?;
        let dimensions: usize = row.get::<_, u32>(5)? as usize;

        // Convert blob to f32 vector (safe: skip malformed blobs)
        if embedding_blob.len() % 4 != 0 || embedding_blob.len() / 4 < dimensions {
            return Ok(("".to_string(), "".to_string(), "".to_string(), 0, Vec::new()));
        }
        let embedding: Vec<f32> = embedding_blob
            .chunks_exact(4)
            .take(dimensions)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok((path, name, library_name, modified, embedding))
    }).ok();

    if let Some(rows) = rows {
        // Two-pass approach: collect all (result, similarity) pairs first,
        // then apply dynamic threshold relative to top score.
        // e5-small produces compressed similarity ranges (0.72–0.88 typical),
        // so a fixed threshold fails — we need adaptive filtering.
        let mut all: Vec<(SearchResult, f32)> = Vec::new();
        for row in rows.flatten() {
            let (path, name, library_name, modified, embedding) = row;
            let sim = cosine_similarity(query_embedding, &embedding);
            if sim > 0.5 { // absolute floor — skip completely irrelevant
                all.push((SearchResult {
                    path, name, library_name, modified,
                    score: sim as f64,
                    match_type: "semantic".to_string(),
                    snippet: None,
                    heading_breadcrumb: None,
                    match_via: None,
                }, sim));
            }
        }

        if !all.is_empty() {
            let top_score = all.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
            // Dynamic threshold: within 3% of top score, minimum 0.75
            let dynamic_thresh = f32::max(0.75, top_score - 0.03);
            for (r, sim) in all {
                if sim >= dynamic_thresh {
                    scored.push((r, sim));
                }
            }
        }
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit as usize);
    scored.into_iter().map(|(r, _)| r).collect()
}

/// Reciprocal Rank Fusion: merges two ranked result lists.
/// RRF_score(d) = Σ 1/(k + rank_i(d))
fn rrf_fuse(list_a: Vec<SearchResult>, list_b: Vec<SearchResult>, k: u32) -> Vec<SearchResult> {
    let mut scores: HashMap<String, (f64, SearchResult)> = HashMap::new();

    for (rank, r) in list_a.into_iter().enumerate() {
        let rrf = 1.0 / (k as f64 + rank as f64 + 1.0);
        scores.entry(r.path.clone()).or_insert((0.0, r)).0 += rrf;
    }

    for (rank, r) in list_b.into_iter().enumerate() {
        let rrf = 1.0 / (k as f64 + rank as f64 + 1.0);
        let path = r.path.clone();
        let mt = r.match_type.clone();
        let entry = scores.entry(path).or_insert((0.0, r));
        entry.0 += rrf;
        if entry.1.match_type == "content" && mt == "semantic" {
            entry.1.match_type = "hybrid".to_string();
        }
    }

    let mut fused: Vec<SearchResult> = scores.into_iter().map(|(_, (score, mut r))| {
        r.score = score;
        r
    }).collect();

    fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    fused
}

// ─── Embedding Storage Commands ────────────────────────────────

/// Store a pre-computed embedding vector for a note (called from JS semantic engine).
#[tauri::command]
pub fn constellation_search_store_embedding(
    app: tauri::AppHandle,
    note_path: String,
    embedding: Vec<f32>,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db.as_ref() {
        let dimensions = embedding.len() as u32;
        // Convert f32 vec to blob (little-endian bytes)
        let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT OR REPLACE INTO note_embeddings (path, embedding, dimensions, cid_cn) VALUES (?1, ?2, ?3, (SELECT cid_cn FROM note_meta WHERE path = ?1))",
            params![note_path, blob, dimensions],
        ).map_err(|e| format!("Failed to store embedding: {}", e))?;
    }
    Ok(())
}

/// Find notes semantically similar to a given note.
#[tauri::command]
pub fn constellation_search_similar(
    app: tauri::AppHandle,
    note_path: String,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search index not initialized")?;

    // Get the note's embedding
    let embedding_blob: Vec<u8> = conn.query_row(
        "SELECT embedding FROM note_embeddings WHERE path = ?1",
        params![note_path],
        |row| row.get(0),
    ).map_err(|_| "Note has no embedding".to_string())?;

    let query_embedding: Vec<f32> = embedding_blob
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    let mut results = semantic_search(conn, &query_embedding, limit.unwrap_or(20));
    // Remove the query note itself from results
    results.retain(|r| r.path != note_path);
    Ok(results)
}

// ─── Universal Categorized Search ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct UniversalSearchResponse {
    pub titles: Vec<SearchResult>,
    pub contents: Vec<SearchResult>,
    pub tags: Vec<SearchResult>,
    pub properties: Vec<SearchResult>,
    pub wikilinks: Vec<SearchResult>,
    pub semantic: Vec<SearchResult>,
}

#[tauri::command]
pub fn constellation_search_universal(
    app: tauri::AppHandle,
    query: String,
    query_embedding: Option<Vec<f32>>,
    limit: Option<u32>,
) -> Result<UniversalSearchResponse, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;

    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => {
            // Same lazy-open pattern as constellation_search — cheap DB open
            // only, never a filesystem walk. See that function for rationale.
            drop(db_guard);
            ensure_search_db_ready(&app)?;
            let state = app.state::<SearchState>();
            let db_guard = state.db.lock().map_err(|e| e.to_string())?;
            return match db_guard.as_ref() {
                Some(c) => execute_universal_search(c, &query, query_embedding.as_deref(), if limit.unwrap_or(0) == 0 { 100000 } else { limit.unwrap() }),
                None => Err("Search index not available".to_string()),
            };
        }
    };

    execute_universal_search(conn, &query, query_embedding.as_deref(), if limit.unwrap_or(0) == 0 { 100000 } else { limit.unwrap() })
}

/// Split query by comma variants: , (Latin) ، (Arabic) 、(CJK)
fn split_multi_terms(query: &str) -> Vec<String> {
    query.split(|c| c == ',' || c == '،' || c == '、')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Deduplicate results by path, keeping highest score
fn dedup_results(mut results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen = std::collections::HashMap::new();
    let mut deduped = Vec::new();
    for r in results.drain(..) {
        let entry = seen.entry(r.path.clone()).or_insert(0.0_f64);
        if r.score > *entry {
            *entry = r.score;
            deduped.retain(|existing: &SearchResult| existing.path != r.path);
            deduped.push(r);
        }
    }
    deduped
}

fn execute_universal_search(conn: &Connection, query: &str, query_embedding: Option<&[f32]>, limit: u32) -> Result<UniversalSearchResponse, String> {
    let terms = split_multi_terms(query);

    let mut all_titles = Vec::new();
    let mut all_contents = Vec::new();
    let mut all_tags = Vec::new();
    let mut all_properties = Vec::new();
    let mut all_wikilinks = Vec::new();

    for term in &terms {
        let normalized = normalize_arabic_for_search(term);
        let raw_lower = term.to_lowercase();

        // Title search: try BOTH original AND normalized (name is stored original,
        // but user might type either form of Arabic)
        all_titles.extend(search_titles(conn, &raw_lower, limit));
        if normalized != raw_lower {
            all_titles.extend(search_titles(conn, &normalized, limit));
        }
        all_contents.extend(search_contents(conn, &normalized, limit));
        // Tags and wikilinks: search both original and normalized
        all_tags.extend(search_tags(conn, &raw_lower, limit));
        if normalized != raw_lower {
            all_tags.extend(search_tags(conn, &normalized, limit));
        }
        all_properties.extend(search_properties(conn, &raw_lower, limit));
        all_wikilinks.extend(search_wikilinks(conn, &raw_lower, limit));
        if normalized != raw_lower {
            all_wikilinks.extend(search_wikilinks(conn, &normalized, limit));
        }
    }

    // 6. SEMANTIC — cosine similarity on stored embeddings (if query embedding provided)
    let semantic = if let Some(qe) = query_embedding {
        let mut results = semantic_search(conn, qe, limit);
        results.truncate(limit as usize);
        results
    } else {
        Vec::new()
    };

    // Deduplicate and truncate
    let mut titles = dedup_results(all_titles); titles.truncate(limit as usize);
    let mut contents = dedup_results(all_contents); contents.truncate(limit as usize);
    let mut tags = dedup_results(all_tags); tags.truncate(limit as usize);
    let mut properties = dedup_results(all_properties); properties.truncate(limit as usize);
    let mut wikilinks = dedup_results(all_wikilinks); wikilinks.truncate(limit as usize);

    Ok(UniversalSearchResponse { titles, contents, tags, properties, wikilinks, semantic })
}

fn search_titles(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let fts_query = format!("name:{}*", query.replace('"', ""));
    let mut stmt = match conn.prepare(
        "SELECT note_meta.path, note_meta.name, note_meta.library_name, note_meta.modified,
                bm25(notes_fts, 10.0, 0.0) as score
         FROM notes_fts
         JOIN note_meta ON notes_fts.rowid = note_meta.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY score
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![fts_query, limit], |row| {
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: row.get::<_, f64>(4)?.abs(),
            snippet: None,
            match_type: "title".to_string(),
            heading_breadcrumb: None,
            match_via: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_contents(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let fts_query = format!("body_text:{}*", query.replace('"', ""));
    let mut stmt = match conn.prepare(
        "SELECT note_meta.path, note_meta.name, note_meta.library_name, note_meta.modified,
                bm25(notes_fts, 0.0, 1.0) as score,
                snippet(notes_fts, 1, '<mark>', '</mark>', '...', 40) as snip
         FROM notes_fts
         JOIN note_meta ON notes_fts.rowid = note_meta.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY score
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![fts_query, limit], |row| {
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: row.get::<_, f64>(4)?.abs(),
            snippet: row.get(5).ok(),
            match_type: "content".to_string(),
            heading_breadcrumb: None,
            match_via: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_tags(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    // Use JSON-quoted match for exact tag: "tagname" in the JSON array
    // This avoids substring false positives (e.g., "id" matching "video")
    let mut stmt = match conn.prepare(
        "SELECT path, name, library_name, modified, tags_json FROM note_meta
         WHERE tags_json LIKE '%\"' || ?1 || '\"%'
         ORDER BY modified DESC
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![query, limit], |row| {
        let tags_json: String = row.get(4)?;
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            snippet: Some(tags_json),
            match_type: "tag".to_string(),
            heading_breadcrumb: None,
            match_via: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_properties(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let mut stmt = match conn.prepare(
        "SELECT path, name, library_name, modified, properties_json FROM note_meta
         WHERE properties_json LIKE '%' || ?1 || '%'
         ORDER BY modified DESC
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![query, limit], |row| {
        let props_json: String = row.get(4)?;
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            snippet: Some(props_json),
            match_type: "property".to_string(),
            heading_breadcrumb: None,
            match_via: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_wikilinks(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let mut stmt = match conn.prepare(
        "SELECT path, name, library_name, modified, outgoing_links_json FROM note_meta
         WHERE outgoing_links_json LIKE '%\"' || ?1 || '\"%'
         ORDER BY modified DESC
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![query, limit], |row| {
        let links_json: String = row.get(4)?;
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            snippet: Some(links_json),
            match_type: "wikilink".to_string(),
            heading_breadcrumb: None,
            match_via: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

/// Targeted FTS re-tokenization for notes whose text contains `needle`.
///
/// Used by the Arabic override CRUD path: when the user pins or removes
/// an override for a surface (say "خليفة"), every note whose body or name
/// mentions that surface has a stale Layer 0 verdict in the on-disk FTS
/// index. This function re-runs the tokenizer over just those rows — so
/// the next `MATCH` query reflects the new override without waiting for
/// a full Universe rebuild.
///
/// Implementation:
/// 1. Arabic-normalize `needle` the same way `index_note` normalizes
///    `body_text` on storage, so the LIKE match catches notes that use
///    any Alef variant, tashkeel, etc.
/// 2. `SELECT rowid, name, body_text FROM note_meta WHERE body_text LIKE ?
///    OR name LIKE ?` — O(N_notes). On 7,600 notes this scans ~10–50 MB of
///    body text; measured sub-100ms. If this ever gets slow, swap to an
///    FTS5 MATCH, but MATCH goes through the tokenizer (which we're trying
///    to refresh) — LIKE on the raw normalized body is the cheapest signal
///    that survives tokenizer changes.
/// 3. For each hit: issue the FTS5 `delete` directive then re-insert. Both
///    operations go through the active tokenizer, so the new `ACTIVE_STORE`
///    override verdict lands in the index.
///
/// Returns the number of rows re-tokenized. Zero is not an error — it
/// just means no indexed note mentions `needle`.
pub fn reindex_notes_matching_text(
    state: &SearchState,
    needle: &str,
) -> Result<u32, String> {
    if needle.trim().is_empty() {
        return Ok(0);
    }
    let needle_normalized = normalize_arabic_for_search(needle);
    // Body and name LIKE patterns: %needle% (case-sensitive; Arabic is not case-bearing).
    let like = format!("%{}%", needle_normalized);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = match db.as_ref() {
        Some(c) => c,
        None => return Ok(0), // No DB yet — nothing to reindex.
    };

    // Gather affected rows first (rowid, name, body_text) so we can issue
    // the delete + insert in a single transaction without holding a prepared
    // statement open across the writes.
    let mut stmt = conn
        .prepare(
            "SELECT rowid, name, body_text
             FROM note_meta
             WHERE body_text LIKE ?1 OR name LIKE ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(i64, String, String)> = stmt
        .query_map(params![like], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    if rows.is_empty() {
        return Ok(0);
    }

    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    let mut count: u32 = 0;
    for (rowid, name, body_text) in &rows {
        // Delete the existing FTS row, then re-insert so the tokenizer
        // runs again with the current ACTIVE_STORE in scope.
        let del = conn.execute(
            "INSERT INTO notes_fts(notes_fts, rowid, name, body_text) VALUES('delete', ?1, ?2, ?3)",
            params![rowid, name, body_text],
        );
        if del.is_err() {
            continue;
        }
        let ins = conn.execute(
            "INSERT INTO notes_fts(rowid, name, body_text) VALUES (?1, ?2, ?3)",
            params![rowid, name, body_text],
        );
        if ins.is_ok() {
            count += 1;
        }
    }
    conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
    Ok(count)
}

/// Return incoming link counts for all notes from the search database.
#[tauri::command]
pub fn constellation_search_link_counts(
    app: tauri::AppHandle,
) -> Result<std::collections::HashMap<String, u32>, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => return Ok(std::collections::HashMap::new()),
    };

    // Initialize counts for all notes
    let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut name_stmt = conn.prepare("SELECT LOWER(name) FROM note_meta")
        .map_err(|e| e.to_string())?;
    let names: Vec<String> = name_stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    for name in &names {
        counts.insert(name.clone(), 0);
    }

    // Scan all outgoing links and count targets
    let mut links_stmt = conn.prepare("SELECT outgoing_links_json FROM note_meta")
        .map_err(|e| e.to_string())?;
    let all_links: Vec<String> = links_stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    for links_json in &all_links {
        if let Ok(targets) = serde_json::from_str::<Vec<String>>(links_json) {
            for target in &targets {
                if let Some(count) = counts.get_mut(&target.to_lowercase()) {
                    *count += 1;
                }
            }
        }
    }

    Ok(counts)
}

// ── M8c — end-to-end override → reindex → FTS token shift ─────────────
//
// These tests exercise the full contract that `add_arabic_override`
// relies on: an override authored in the active Universe, followed by
// a `reindex_arabic_overrides` call, must cause `notes_fts` to contain
// the overridden stem for every note that mentions the original surface.
//
// They are unit-level in the sense that they do NOT require the Tauri
// AppHandle or a running app — they drive `reindex_notes_matching_text`
// directly against a tempfile SQLite DB initialized via `init_db`.
// This matches the M8c scope split (ship the integration test without
// the Settings → Debug UI scorecard, which is tracked as M8d).

#[cfg(test)]
mod tests_m8c {
    use super::*;
    use crate::arabic::overrides::{
        self, OverrideStore, UserOverride, TEST_OVERRIDE_MUTEX,
    };
    use crate::arabic::PartOfSpeech;
    use std::sync::MutexGuard;

    /// Serializes ACTIVE_STORE mutations across every test in this
    /// suite AND the tests in `arabic::overrides::tests`. Without this,
    /// cargo test's default multi-thread runner would race the global
    /// override registry between suites.
    struct OverrideTestGuard {
        _lock: MutexGuard<'static, ()>,
    }

    impl OverrideTestGuard {
        fn new() -> Self {
            let lock = TEST_OVERRIDE_MUTEX
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            overrides::clear_active();
            Self { _lock: lock }
        }
    }

    impl Drop for OverrideTestGuard {
        fn drop(&mut self) {
            overrides::clear_active();
        }
    }

    /// Author a unique temp-dir path for one test run. Each test uses
    /// a nanosecond-stamped directory so concurrent test workers don't
    /// collide on the same SQLite file.
    fn unique_tmp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "constellation_m8c_{}_{}",
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    /// Build a fresh `SearchState` backed by a tempfile DB, seeded
    /// with one `note_meta` row whose `body_text` is `body`.
    ///
    /// The `note_meta_ai` trigger fires automatically on INSERT, so
    /// after this call `notes_fts` already holds the tokenized form
    /// of `body` under the current `ACTIVE_STORE` (which the caller
    /// controls — usually empty to capture the pre-override baseline).
    ///
    /// **Production parity**: `index_note` at search.rs:628 pre-normalises
    /// `plain_body` via `normalize_arabic_for_search` *before* the INSERT,
    /// so every `body_text` row on disk is already in normalised form
    /// (Teh marbuta → Heh, Alef variants collapsed, diacritics stripped).
    /// `reindex_notes_matching_text` relies on this invariant — it
    /// normalises the needle and does `body_text LIKE %needle_normalised%`,
    /// so raw-Arabic body rows would silently fail the LIKE match. The
    /// seed mirrors production by normalising here too.
    fn seeded_state(dir: &Path, note_path: &str, body: &str) -> SearchState {
        std::fs::create_dir_all(dir).expect("mkdir tempdir");
        let db_path = dir.join("search.db");
        let conn = init_db(&db_path).expect("init_db");
        let body_normalised = super::normalize_arabic_for_search(body);
        conn.execute(
            "INSERT INTO note_meta(path, name, library_name, modified, body_text) \
             VALUES (?1, ?2, ?3, 0, ?4)",
            params![note_path, "test_note", "testlib", body_normalised],
        )
        .expect("seed note_meta");
        SearchState {
            db: std::sync::Mutex::new(Some(conn)),
        }
    }

    /// Count FTS5 rows that MATCH `query` against the current state.
    /// Takes out a read lock and runs a single COUNT query.
    fn fts_match_count(state: &SearchState, query: &str) -> u32 {
        let db = state.db.lock().expect("db lock");
        let conn = db.as_ref().expect("conn present");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH ?1",
                params![query],
                |r| r.get(0),
            )
            .expect("MATCH count query");
        count as u32
    }

    /// Tear down a test DB directory (file + WAL/SHM siblings + parent).
    fn cleanup(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Build a single-lemma sovereign override for tests. Uses the
    /// Latin sentinel `pinnedteststem` as the lemma — guaranteed not
    /// to collide with any Arabic analyzer verdict, so MATCH for the
    /// sentinel cleanly distinguishes pre-override from post-override
    /// FTS state.
    fn sentinel_override(surface: &str, lemma: &str) -> UserOverride {
        UserOverride {
            surface: surface.to_string(),
            lemma: lemma.to_string(),
            root: String::new(),
            pattern_label: "user:ProperNoun".to_string(),
            pos: PartOfSpeech::ProperNoun,
            note: String::new(),
            created_at: String::new(),
        }
    }

    /// The headline M8c contract: authoring an override then running
    /// `reindex_notes_matching_text` flips every mentioning note's
    /// FTS row from the default stem to the override's lemma. If this
    /// ever regresses, the Settings UI "pin this word" button becomes
    /// a silent no-op on the existing index — exactly the failure M8c
    /// is written to catch.
    ///
    /// **Surface choice**: `خليفة` (with ta-marbuta) is the canonical
    /// spelling. The test specifically uses a surface with ta-marbuta
    /// because an earlier revision of `normalize_arabic_for_search`
    /// folded ة → ه, which silently disagreed with the override
    /// store's key normalizer (which never folds) — breaking exactly
    /// the reindex contract this test exists to guard. The test is
    /// preserved with its original linguistically-correct surface so
    /// any future regression that reintroduces the fold trips here
    /// before it reaches users.
    #[test]
    fn override_and_reindex_flips_fts_token_set() {
        let _g = OverrideTestGuard::new();
        let dir = unique_tmp_dir("flip");

        // Body contains the target surface + one unrelated word so the
        // row still matches some query even after override rewrites the
        // target's stem.
        let state = seeded_state(&dir, "/notes/khalifa.md", "خليفة راشد");

        let sentinel = "pinnedteststem";

        // 1) Pre-override: sentinel is nowhere in the index.
        assert_eq!(
            fts_match_count(&state, sentinel),
            0,
            "sentinel must be absent from FTS before any override"
        );

        // 2) Install the override.
        let mut store = OverrideStore::new();
        store.insert(sentinel_override("خليفة", sentinel));
        overrides::set_active(store);

        // 3) Critical contract: override alone doesn't retroactively
        //    rewrite existing FTS rows. The row was tokenized BEFORE
        //    we activated, so the sentinel stem isn't yet present.
        //    This is exactly why `reindex_notes_matching_text` exists.
        assert_eq!(
            fts_match_count(&state, sentinel),
            0,
            "override in ACTIVE_STORE alone must not retroactively update FTS"
        );

        // 4) Run the reindex helper — the Tauri command's inner body.
        let reindexed = reindex_notes_matching_text(&state, "خليفة")
            .expect("reindex must succeed");
        assert_eq!(
            reindexed, 1,
            "exactly the one mentioning row should be re-tokenized"
        );

        // 5) Post-reindex: the override's lemma is now in the FTS index
        //    and MATCH finds it.
        assert_eq!(
            fts_match_count(&state, sentinel),
            1,
            "sentinel stem must be present in FTS after reindex"
        );

        cleanup(&dir);
    }

    /// A surface that no note mentions must return 0 re-tokenizations
    /// without error — the `add_arabic_override` + reindex chain is
    /// idempotent for words that haven't been indexed yet, which is
    /// the common case for the user adding a forward-looking override.
    #[test]
    fn reindex_returns_zero_when_no_notes_match() {
        let _g = OverrideTestGuard::new();
        let dir = unique_tmp_dir("nomatch");

        // Body contains some other Arabic words — not the target.
        let state = seeded_state(&dir, "/notes/other.md", "كتاب مفيد");

        let count = reindex_notes_matching_text(&state, "خليفة")
            .expect("reindex must succeed on zero-match needle");
        assert_eq!(count, 0, "no mentioning rows → zero re-tokenizations");

        cleanup(&dir);
    }

    /// Empty / whitespace needle must short-circuit to 0 without
    /// issuing the expensive `body_text LIKE %%` scan. Guards the
    /// Settings UI against an accidental empty-string dispatch
    /// triggering a full-table scan on a 7,600-note Universe.
    #[test]
    fn reindex_empty_needle_short_circuits() {
        let _g = OverrideTestGuard::new();
        let dir = unique_tmp_dir("empty");
        let state = seeded_state(&dir, "/notes/x.md", "some body");

        assert_eq!(
            reindex_notes_matching_text(&state, "")
                .expect("empty needle must not error"),
            0
        );
        assert_eq!(
            reindex_notes_matching_text(&state, "   ")
                .expect("whitespace needle must not error"),
            0
        );

        cleanup(&dir);
    }

    /// The reindex walks `body_text LIKE %needle%` — every matching
    /// row gets re-tokenized in one IMMEDIATE transaction. Multiple
    /// rows containing the same surface must all flip to the new
    /// sentinel stem, not just the first one.
    ///
    /// See `override_and_reindex_flips_fts_token_set` for the
    /// rationale on using a ta-marbuta-bearing surface deliberately.
    #[test]
    fn reindex_updates_all_matching_rows_in_one_pass() {
        let _g = OverrideTestGuard::new();
        let dir = unique_tmp_dir("multi");
        std::fs::create_dir_all(&dir).expect("mkdir");
        let db_path = dir.join("search.db");
        let conn = init_db(&db_path).expect("init_db");

        // Three notes all mentioning خليفة, one unrelated note.
        // Bodies are pre-normalised to match production `index_note`
        // behaviour (see the note on `seeded_state` for full rationale).
        for (path, body) in [
            ("/notes/a.md", "خليفة راشد"),
            ("/notes/b.md", "عمر بن خليفة"),
            ("/notes/c.md", "كتاب عن خليفة"),
            ("/notes/d.md", "كتاب مفيد"),
        ] {
            let body_normalised = super::normalize_arabic_for_search(body);
            conn.execute(
                "INSERT INTO note_meta(path, name, library_name, modified, body_text) \
                 VALUES (?1, ?2, ?3, 0, ?4)",
                params![path, "note", "testlib", body_normalised],
            )
            .expect("seed");
        }
        let state = SearchState {
            db: std::sync::Mutex::new(Some(conn)),
        };

        let sentinel = "bulktestsentinel";
        let mut store = OverrideStore::new();
        store.insert(sentinel_override("خليفة", sentinel));
        overrides::set_active(store);

        let reindexed = reindex_notes_matching_text(&state, "خليفة")
            .expect("reindex");
        assert_eq!(
            reindexed, 3,
            "all three mentioning rows (not the unrelated one) should be re-tokenized"
        );
        assert_eq!(
            fts_match_count(&state, sentinel),
            3,
            "every mentioning row must now contain the sentinel stem"
        );

        cleanup(&dir);
    }
}

// ─── M12 (lexicon wire-up) unit tests ──────────────────────────
//
// These tests pin the decision boundary of `expanded_match_query`:
// when does `lexical_search` take the cross-lingual bridge vs.
// fall back to today's `word*` prefix match?
//
// The bridge only kicks in when `detect_source_lang` succeeds AND
// `build_match_expr` produces an expression with " OR " (i.e. the
// lemma actually has translations or synonyms in the corpus). Any
// other shape — unknown language, punctuation-only, or in-corpus
// lemma with zero bridge edges — returns None so prefix matching
// still runs. Regressing that surface would silently degrade recall
// on every query the production corpus doesn't cover.

#[cfg(test)]
mod tests_m12 {
    use super::expanded_match_query;

    #[test]
    fn known_english_word_bridges_cross_lingually() {
        // "tree" is concept c:tree in lexicon_v1.tsv with rich
        // coverage across all 15 languages.
        let exp = expanded_match_query("tree")
            .expect("tree must bridge — it's in the production corpus");
        assert!(
            exp.match_expr.contains(" OR "),
            "expected OR-joined expression, got {:?}",
            exp.match_expr,
        );
        // Spot-check that the Arabic translation actually made it in.
        assert!(
            exp.match_expr.contains("شجرة"),
            "expected ar:شجرة in expansion, got {:?}",
            exp.match_expr,
        );
        // And that the source lemma is preserved.
        assert!(
            exp.match_expr.contains("\"tree\""),
            "expected source lemma 'tree' in expansion, got {:?}",
            exp.match_expr,
        );
    }

    #[test]
    fn known_arabic_word_bridges_to_english() {
        // Reverse direction — Arabic lemma should pull in English
        // plus other languages via the same concept node.
        let exp = expanded_match_query("شجرة")
            .expect("شجرة must bridge — it's ar: on concept c:tree");
        assert!(
            exp.match_expr.contains(" OR "),
            "expected OR-joined expression, got {:?}",
            exp.match_expr,
        );
        assert!(
            exp.match_expr.contains("\"tree\""),
            "expected en:tree in expansion, got {:?}",
            exp.match_expr,
        );
    }

    #[test]
    fn unknown_word_falls_back_to_none() {
        // "quasar" is Latin script so detect_source_lang returns
        // Some(Lang::En), but the lemma isn't in the corpus. Expansion
        // echoes only the source lemma → single-term expr → no " OR " →
        // we return None so the caller falls back to `quasar*`.
        assert!(
            expanded_match_query("quasar").is_none(),
            "unknown words must NOT take the bridge — prefix fallback \
             preserves today's recall for out-of-corpus queries"
        );
    }

    #[test]
    fn punctuation_only_returns_none() {
        // detect_source_lang returns None for strings with no
        // strong-script characters, so the whole bridge short-circuits.
        assert!(expanded_match_query("   !!!").is_none());
        assert!(expanded_match_query("").is_none());
        assert!(expanded_match_query("123").is_none());
    }

    #[test]
    fn proper_noun_not_in_corpus_falls_back() {
        // Any well-formed English word not mapped to a concept returns
        // None and falls through to prefix matching. The corpus has
        // scaled well past its original 49-concept seed (now ~20K
        // concepts across 499 shards), so these assertions use proper
        // nouns and invented strings guaranteed to stay out of the
        // natural-language lexicon: "Anthropic" (company name, not a
        // concept) and "Xzyqwop" (invented, zero-collision).
        assert!(expanded_match_query("Anthropic").is_none());
        assert!(expanded_match_query("Xzyqwop").is_none());
    }
}

// ─── M13 tests ────────────────────────────────────────────────────────────────
//
// `find_match_via` powers the cross-lingual result badge. It scans the
// FTS5 `snippet()` output — an HTML fragment with `<mark>…</mark>` around
// every matched token — and returns the first marked region whose text
// is a known bridge term. These tests pin down three properties:
//
//   1. Only `<mark>`-anchored matches count. A bridge term that happens
//      to appear in the unmarked context window is ignored — otherwise
//      we'd badge results where the real hit was on the source lemma.
//   2. Case is folded on both sides. FTS5 emits snippets with the
//      document's original casing; bridge terms come from the corpus
//      pre-lowercased by `expanded_match_query`.
//   3. First mark wins. If a note happens to contain both a source-lang
//      and a bridge-lang token, the first marked region decides the
//      badge — deterministic, and the source case was already short-
//      circuited by caller logic (title-hit test + same-lang filter in
//      `expanded_match_query`).
//
// The `bridge_terms` tests on `LexicalExpansion` verify that the source
// language is filtered out. That filter is what prevents "tree" matching
// a note containing "trees" from earning a spurious "via trees" badge.

#[cfg(test)]
mod tests_m13 {
    use super::{expanded_match_query, find_match_via};

    #[test]
    fn mark_containing_bridge_term_returns_it() {
        // Baseline — the mark's text exactly equals a bridge term.
        let snippet = "a note about <mark>شجرة</mark> in the garden";
        let bridge = vec!["شجرة".to_string()];
        assert_eq!(
            find_match_via(snippet, &bridge),
            Some("شجرة".to_string()),
        );
    }

    #[test]
    fn source_lemma_match_returns_none() {
        // Only the source lemma is marked — no bridge term visible.
        // The caller already filtered source-lang terms out of
        // `bridge_terms_lower`, so the slice passed here contains only
        // non-source languages. A marked region that doesn't match any
        // of them means the hit came via the user's own lemma.
        let snippet = "planting a <mark>tree</mark> today";
        let bridge = vec!["شجرة".to_string(), "árbol".to_string()];
        assert!(find_match_via(snippet, &bridge).is_none());
    }

    #[test]
    fn first_mark_wins_when_multiple_bridges_present() {
        // Two marks, both bridge terms. The earlier one decides the
        // badge — keeps rendering deterministic across reruns.
        let snippet = "<mark>شجرة</mark> and <mark>árbol</mark> mean the same";
        let bridge = vec!["شجرة".to_string(), "árbol".to_string()];
        assert_eq!(
            find_match_via(snippet, &bridge),
            Some("شجرة".to_string()),
        );
    }

    #[test]
    fn unmarked_bridge_occurrence_is_ignored() {
        // The word "شجرة" appears in context but isn't inside `<mark>`.
        // The real FTS match was on "tree" (the source lemma), so the
        // badge must NOT fire — anchoring on `<mark>` is what makes
        // this behavior correct.
        let snippet = "planting a <mark>tree</mark> — شجرة in Arabic";
        let bridge = vec!["شجرة".to_string()];
        assert!(
            find_match_via(snippet, &bridge).is_none(),
            "bridge term outside <mark> must not earn a badge"
        );
    }

    #[test]
    fn case_is_folded_on_snippet_side() {
        // FTS5 keeps the document's original casing inside `<mark>`.
        // Bridge terms are pre-lowercased by `expanded_match_query`,
        // so the scan lowercases the marked region before comparing.
        let snippet = "the <mark>Árbol</mark> is old";
        let bridge = vec!["árbol".to_string()];
        assert_eq!(
            find_match_via(snippet, &bridge),
            Some("árbol".to_string()),
        );
    }

    #[test]
    fn empty_bridge_terms_returns_none_fast() {
        // Common case when expansion only produced same-language
        // synonyms: `bridge_terms_lower` is empty and no badge can
        // ever fire. Early-out avoids the scan entirely.
        let snippet = "a <mark>tree</mark> stands tall";
        assert!(find_match_via(snippet, &[]).is_none());
    }

    #[test]
    fn snippet_without_marks_returns_none() {
        // Defensive — in practice we only call `find_match_via` when
        // FTS5 returned a body hit, so there's always ≥1 `<mark>`. But
        // if the snippet column was NULL or malformed, we must not panic.
        let bridge = vec!["شجرة".to_string()];
        assert!(find_match_via("", &bridge).is_none());
        assert!(find_match_via("no marks here at all", &bridge).is_none());
    }

    #[test]
    fn unterminated_mark_breaks_out_cleanly() {
        // Malformed HTML — a `<mark>` without a closing tag. We break
        // the loop at the open tag rather than scanning to EOF. No panic,
        // no false positive.
        let snippet = "a <mark>tree and more text with no close";
        let bridge = vec!["tree".to_string()];
        assert!(find_match_via(snippet, &bridge).is_none());
    }

    #[test]
    fn partial_mark_content_match_does_not_badge() {
        // The mark contains a bridge term as a substring but not as
        // the whole marked region. Reject — the FTS match unit is the
        // whole token, and badging a prefix would be incoherent.
        let snippet = "the <mark>árbol-shaped</mark> pattern";
        let bridge = vec!["árbol".to_string()];
        assert!(
            find_match_via(snippet, &bridge).is_none(),
            "bridge term must be the full marked region, not a substring"
        );
    }

    #[test]
    fn english_expansion_excludes_english_from_bridge_terms() {
        // `expanded_match_query("tree")` produces a MATCH expr with
        // English inflections AND cross-lingual translations. The
        // bridge_terms set must contain ONLY the cross-lingual side —
        // otherwise a match on "trees" (plural) would earn a "via trees"
        // badge, which is nonsense for an English-speaking user.
        let exp = expanded_match_query("tree").expect("tree must bridge");
        assert!(
            !exp.bridge_terms_lower.is_empty(),
            "expected cross-lingual bridge terms, got empty set"
        );
        for term in &exp.bridge_terms_lower {
            assert!(
                term != "tree" && term != "trees",
                "English terms must be filtered from bridge_terms, found {:?}",
                term,
            );
        }
        // And the Arabic translation should be present.
        assert!(
            exp.bridge_terms_lower.iter().any(|t| t == "شجرة"),
            "expected 'شجرة' in bridge_terms_lower, got {:?}",
            exp.bridge_terms_lower,
        );
    }

    #[test]
    fn arabic_expansion_excludes_arabic_from_bridge_terms() {
        // Reverse direction — Arabic source, English should appear in
        // bridge_terms and Arabic lemmas should not.
        let exp = expanded_match_query("شجرة").expect("شجرة must bridge");
        for term in &exp.bridge_terms_lower {
            assert!(
                term != "شجرة",
                "Arabic source must be filtered from bridge_terms, found {:?}",
                term,
            );
        }
        assert!(
            exp.bridge_terms_lower.iter().any(|t| t == "tree"),
            "expected 'tree' in bridge_terms_lower for Arabic source, got {:?}",
            exp.bridge_terms_lower,
        );
    }

    #[test]
    fn bridge_terms_are_pre_lowercased() {
        // `find_match_via` assumes `bridge_terms_lower` is already
        // lowercased — it does `to_lowercase()` only on the snippet
        // side, per row. Verify the contract holds at the source.
        let exp = expanded_match_query("tree").expect("tree must bridge");
        for term in &exp.bridge_terms_lower {
            assert_eq!(
                term.as_str(),
                term.to_lowercase().as_str(),
                "bridge term must be pre-lowercased, found mixed case: {:?}",
                term,
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// M14-bench — `lexical_search` end-to-end latency bench.
// ─────────────────────────────────────────────────────────────────────
//
// Run with:
//
// ```bash
// cargo test --lib --release search::m14_bench -- --ignored --nocapture
// ```
//
// # What this measures
//
// One `#[test] #[ignore]` function that seeds a tempfile SQLite DB with
// ~100 bench notes (English-only + Arabic-only + mixed bodies centred
// on lexicon-covered concepts), then times `lexical_search` across
// three query shapes:
//
//   (a) known-word → bridges:   "tree", "كتاب", "livre"
//   (b) unknown-word → prefix:  "quasar", "Constellation", "xyzzy"
//   (c) Arabic-only (native):   "شجرة", "معرفة"
//
// Per query: warmup 20 calls, then SAMPLES=500 timed calls via
// `Instant::now()` bracketing each invocation. The harness reports
// mean / p50 / p95 / p99 / max per shape.
//
// # Why it matters
//
// M12-wire (§ 43) rewired `lexical_search` to try cross-language
// expansion via the Lexical Bridge before falling back to prefix
// matching. The risk: Arabic-only queries on Arabic-only corpora used
// to run a single-term FTS5 MATCH; now they may run a 15-branch OR-join
// that contributes zero hits against the Arabic-only FTS rows. If the
// new OR-joined MATCH is measurably slower the regression is silent at
// the user-facing level (same result set, same ranking) but pays a
// latency tax on every keystroke of Arabic search.
//
// This bench is the **regression gate**: a hard p99 budget on shape
// (c), plus informational stats on (a) and (b). Trips on the next opt-in
// `--ignored` run if the post-M12-wire pipeline regresses.
//
// # Budgets
//
// * Shape (a) p99 <  10 ms — bridged path, adds OR branches.
// * Shape (b) p99 <  10 ms — prefix fallback, identical shape to pre-M12.
// * Shape (c) p99 <  10 ms — the critical non-regression gate.
//
// These are absolute budgets on a 100-note corpus; typical observed
// latency on a warm DB is expected to be 1–2 orders of magnitude below
// the budget (sub-ms). The headroom absorbs CI variance without hiding
// a genuine regression — a 10× slowdown would still trip.
//
// # Scope note
//
// Not gated on `cargo test --lib` — only on an explicit `--ignored`
// invocation, matching `arabic::bench::m9_bench` and `lexicon::bench::m12_bench`.
// The tempfile corpus is cleaned up at the end; runs leave no state.

#[cfg(test)]
mod m14_bench {
    use super::{init_db, lexical_search, normalize_arabic_for_search};
    use rusqlite::params;
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    /// English-only bench bodies. Each centres on one lexicon-covered
    /// concept so the bridged path (shape (a)) actually materialises
    /// hits rather than producing a zero-result MATCH. Filler words
    /// around the anchor keep the FTS tokens realistic (not a single
    /// lemma stuffed 100 times).
    const EN_BODIES: &[(&str, &str)] = &[
        ("en_tree_1", "The old tree behind the house casts a long shadow at dusk."),
        ("en_tree_2", "She planted a tree for every birthday; the garden is thirty trees strong."),
        ("en_tree_3", "Oak is the tallest tree on this hillside."),
        ("en_book_1", "The library keeps every book in alphabetical order by author."),
        ("en_book_2", "He borrowed a book on celestial navigation for the weekend."),
        ("en_book_3", "A book about knowledge is a book about everything."),
        ("en_house_1", "The house on the corner has been empty since spring."),
        ("en_house_2", "They turned the old house into a small café with a garden."),
        ("en_water_1", "Water from the mountain spring is cold even in July."),
        ("en_water_2", "The water tasted faintly of iron, but it was safe."),
        ("en_knowledge_1", "Knowledge is not a stockpile; it is a working relationship with the world."),
        ("en_knowledge_2", "His knowledge of maps outweighed his knowledge of the road itself."),
        ("en_language_1", "A new language rewrites the shape of your thought."),
        ("en_language_2", "Every language carries the fingerprints of its speakers."),
        ("en_peace_1", "Peace is a practice, not a treaty."),
        ("en_truth_1", "Truth has a simple habit of outlasting the convenient."),
        ("en_love_1", "Love, like writing, rewards the patient over the clever."),
        ("en_time_1", "Time is the only currency you spend before you earn it."),
        ("en_day_1", "A quiet day beside a clear river is a reset."),
        ("en_night_1", "Night in the desert is colder than you expect."),
        ("en_learn_1", "We learn best when we teach, and teach best when we doubt."),
        ("en_idea_1", "An idea held lightly goes further than an idea held tightly."),
        ("en_fire_1", "Fire, water, wood, and wind — the elements of every story."),
        ("en_door_1", "The door to the garden had been oiled but never fixed."),
        ("en_city_1", "The city at midnight is another city entirely."),
        ("en_food_1", "Good food and a long table — nothing else required."),
        ("en_bread_1", "Bread baked that morning at the corner bakery."),
        ("en_earth_1", "The earth under the old tree was cool and damp."),
        ("en_eat_1", "They would eat in silence, and then the stories would begin."),
        ("en_hear_1", "I could hear the tree creaking in the wind all night."),
        ("en_see_1", "Look at the tree: it has stood longer than most families."),
        ("en_read_1", "To read a book is to borrow another person's attention."),
        ("en_write_1", "Writing a book is a slow tree, grown one ring per year."),
        ("en_student_1", "A student asks the teacher how to ask better questions."),
        ("en_teacher_1", "The teacher opened the book and then closed it again."),
        ("en_big_1", "The big tree at the centre of the village."),
        ("en_good_1", "Good water, good bread, good company."),
        ("en_beautiful_1", "A beautiful tree in winter has nothing to hide."),
        ("en_important_1", "What is important is rarely what is urgent."),
        ("en_mixed_1", "A tree, a book, a door, a river — the catalog of a day."),
    ];

    /// Arabic-only bench bodies. Same anchor-concept pattern as EN_BODIES.
    /// These drive shape (c) — an Arabic query hitting Arabic rows with
    /// the bridged MATCH clause carrying zero-hit cross-lingual branches.
    const AR_BODIES: &[(&str, &str)] = &[
        ("ar_tree_1", "الشجرة القديمة خلف البيت تلقي ظلا طويلا عند الغروب."),
        ("ar_tree_2", "زرعت شجرة في كل عيد ميلاد؛ الحديقة الآن ثلاثون شجرة."),
        ("ar_tree_3", "البلوط هو أطول شجرة على هذا التل."),
        ("ar_book_1", "المكتبة تحتفظ بكل كتاب مرتب أبجديا حسب المؤلف."),
        ("ar_book_2", "استعار كتاب عن الملاحة الفلكية لعطلة نهاية الأسبوع."),
        ("ar_book_3", "كتاب عن المعرفة هو كتاب عن كل شيء."),
        ("ar_house_1", "البيت في الزاوية فارغ منذ الربيع."),
        ("ar_house_2", "حولوا البيت القديم إلى مقهى صغير بحديقة."),
        ("ar_water_1", "الماء من نبع الجبل بارد حتى في يوليو."),
        ("ar_water_2", "طعم الماء كان فيه نبرة من الحديد، لكنه كان آمنا."),
        ("ar_knowledge_1", "المعرفة ليست مخزونا؛ بل علاقة عمل مع العالم."),
        ("ar_knowledge_2", "معرفته بالخرائط فاقت معرفته بالطريق نفسه."),
        ("ar_language_1", "اللغة الجديدة تعيد رسم شكل الفكر."),
        ("ar_language_2", "كل لغة تحمل بصمات أهلها."),
        ("ar_peace_1", "السلام ممارسة، لا معاهدة."),
        ("ar_truth_1", "للحقيقة عادة بسيطة: أن تبقى بعد أن يذهب ما هو مريح."),
        ("ar_love_1", "الحب، مثل الكتابة، يكافئ الصبور أكثر من الذكي."),
        ("ar_time_1", "الوقت هو العملة الوحيدة التي تنفقها قبل أن تكسبها."),
        ("ar_day_1", "يوم هادئ بجوار نهر صاف هو إعادة ضبط."),
        ("ar_night_1", "الليل في الصحراء أبرد مما تتوقع."),
        ("ar_learn_1", "نتعلم بشكل أفضل حين نعلم، ونعلم بشكل أفضل حين نشك."),
        ("ar_idea_1", "فكرة ممسكة بخفة تذهب أبعد من فكرة ممسكة بشدة."),
        ("ar_fire_1", "نار، ماء، خشب، ريح — عناصر كل قصة."),
        ("ar_door_1", "باب الحديقة كان مزيتا لكن لم يصلح."),
        ("ar_city_1", "المدينة في منتصف الليل مدينة أخرى تماما."),
        ("ar_food_1", "طعام جيد وطاولة طويلة — لا شيء آخر مطلوب."),
        ("ar_bread_1", "الخبز المخبوز في ذلك الصباح عند المخبز المجاور."),
        ("ar_earth_1", "الأرض تحت الشجرة القديمة كانت باردة ورطبة."),
        ("ar_eat_1", "كانوا يأكلون في صمت، ثم تبدأ القصص."),
        ("ar_hear_1", "كنت أسمع الشجرة تصرّ في الريح طوال الليل."),
        ("ar_see_1", "انظر إلى الشجرة: لقد صمدت أطول من معظم العائلات."),
        ("ar_read_1", "قراءة كتاب هي استعارة انتباه شخص آخر."),
        ("ar_write_1", "كتابة كتاب شجرة بطيئة، تنمو حلقة واحدة في السنة."),
        ("ar_student_1", "الطالب يسأل المعلم كيف يسأل أسئلة أفضل."),
        ("ar_teacher_1", "فتح المعلم الكتاب ثم أغلقه مرة أخرى."),
        ("ar_big_1", "الشجرة الكبيرة في منتصف القرية."),
        ("ar_good_1", "ماء جيد، خبز جيد، رفقة جيدة."),
        ("ar_beautiful_1", "شجرة جميلة في الشتاء ليس لديها ما تخفيه."),
        ("ar_important_1", "المهم نادرا ما يكون العاجل."),
        ("ar_mixed_1", "شجرة، كتاب، باب، نهر — فهرس يوم."),
    ];

    /// Mixed-language bench bodies — one concept word in two languages
    /// inside a single body. These stress-test the tokenizer-hand-off
    /// between the Arabic Light10 stemmer and the English path.
    const MIXED_BODIES: &[(&str, &str)] = &[
        ("mix_1", "A tree that grows slowly becomes a long شجرة. The old becomes the ancient."),
        ("mix_2", "Every book in the كتاب is a window to somewhere else."),
        ("mix_3", "Water is ماء — the same molecule, a different word."),
        ("mix_4", "Knowledge is معرفة — working relationship with the world."),
        ("mix_5", "The house is بيت, the home is also بيت — Arabic is generous that way."),
        ("mix_6", "Peace (سلام) costs attention. Peace is practice."),
        ("mix_7", "Truth (الحقيقة) outlasts the convenient."),
        ("mix_8", "Language (لغة) rewrites the shape of thought."),
        ("mix_9", "A student (طالب) asks the teacher (معلم) better questions."),
        ("mix_10", "Fire (نار), water (ماء), wood, wind — elements of every story."),
        ("mix_11", "An idea (فكرة) held lightly travels further."),
        ("mix_12", "City (مدينة) at midnight is another city entirely."),
        ("mix_13", "Food (طعام), bread (خبز), good company."),
        ("mix_14", "Day (يوم) is quiet beside a clear river."),
        ("mix_15", "Night (ليل) in the desert is colder than you expect."),
        ("mix_16", "A door (باب) to the garden, oiled but not fixed."),
        ("mix_17", "To read (قراءة) a book is to borrow another person's attention."),
        ("mix_18", "To write (كتابة) a book is a slow tree, one ring per year."),
        ("mix_19", "Earth (أرض), tree (شجرة), fire (نار), water (ماء) — the elements."),
        ("mix_20", "Learn (تعلم) by teaching; teach (تعليم) by doubting."),
    ];

    fn unique_tmp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "constellation_m14_bench_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    /// Build a fresh tempfile SQLite DB, run `init_db` to register the
    /// constellation tokenizer + create the FTS schema, then insert
    /// all bench bodies. Each INSERT fires `note_meta_ai` which
    /// populates `notes_fts` with the tokenised body — so at return
    /// time the DB is fully ready for `lexical_search`.
    fn seed_bench_corpus() -> (PathBuf, rusqlite::Connection) {
        let dir = unique_tmp_dir();
        std::fs::create_dir_all(&dir).expect("mkdir bench tempdir");
        let db_path = dir.join("search.db");
        let conn = init_db(&db_path).expect("init_db");

        let all = EN_BODIES.iter().chain(AR_BODIES.iter()).chain(MIXED_BODIES.iter());
        for (name, body) in all {
            let body_normalised = normalize_arabic_for_search(body);
            let note_path = format!("/seed/bench/{}.md", name);
            conn.execute(
                "INSERT INTO note_meta(path, name, library_name, modified, body_text) \
                 VALUES (?1, ?2, 'bench', 0, ?3)",
                params![note_path, name, body_normalised],
            )
            .expect("seed insert");
        }
        (dir, conn)
    }

    /// Percentile by rank on a sorted slice. No interpolation — the
    /// sample size (500) is large enough that bucket choice dominates
    /// and mirrors `lexicon::bench::m12_bench`.
    fn percentile(sorted: &[u64], p: f64) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let idx = ((sorted.len() as f64) * p).clamp(0.0, (sorted.len() - 1) as f64) as usize;
        sorted[idx]
    }

    /// Warmup + sample harness around `lexical_search`. Returns a
    /// sorted vector of per-call nanosecond latencies so the caller can
    /// derive percentiles.
    fn measure(
        conn: &rusqlite::Connection,
        query: &str,
        warmup: usize,
        samples: usize,
    ) -> Vec<u64> {
        for _ in 0..warmup {
            black_box(lexical_search(conn, query, 25));
        }
        let mut ns: Vec<u64> = Vec::with_capacity(samples);
        for _ in 0..samples {
            let t = Instant::now();
            let r = lexical_search(conn, query, 25);
            ns.push(t.elapsed().as_nanos() as u64);
            black_box(r);
        }
        ns.sort_unstable();
        ns
    }

    /// Print a compact per-shape stat line. Kept terse so the full
    /// three-shape report fits on one terminal height without scroll.
    fn report_stats(label: &str, samples: &[u64], hits: usize) {
        let sum: u128 = samples.iter().map(|&n| n as u128).sum();
        let mean_ns = (sum / samples.len() as u128) as u64;
        let p50 = percentile(samples, 0.50);
        let p95 = percentile(samples, 0.95);
        let p99 = percentile(samples, 0.99);
        let max = *samples.last().unwrap_or(&0);
        println!(
            "  {label:<28} hits={hits:<3} mean={mean_ns:>8} ns  p50={p50:>8}  p95={p95:>8}  p99={p99:>8}  max={max:>8}"
        );
        println!(
            "  {:<28}            mean={:.2} µs   p50={:.2}      p95={:.2}      p99={:.2}      max={:.2}",
            "",
            mean_ns as f64 / 1_000.0,
            p50 as f64 / 1_000.0,
            p95 as f64 / 1_000.0,
            p99 as f64 / 1_000.0,
            max as f64 / 1_000.0,
        );
    }

    /// M14 end-to-end bench. Opt-in only: `--ignored`.
    ///
    /// The budget constants are expressed in nanoseconds so the assert
    /// message is human-readable when a regression trips it. 10 ms p99
    /// is generous for a 100-note corpus — typical observed latencies
    /// on a warm DB are sub-millisecond, so a tripped assert indicates
    /// a real shape-change in the hot path (unwanted full-table scan,
    /// per-call allocation storm, contention on a global lock, etc.)
    /// rather than CI variance.
    const BUDGET_P99_NS: u64 = 10_000_000; // 10 ms

    #[test]
    #[ignore]
    fn m14_bench() {
        println!("\n=== M14 Bench — lexical_search end-to-end ===\n");

        let (dir, conn) = seed_bench_corpus();
        let note_count = EN_BODIES.len() + AR_BODIES.len() + MIXED_BODIES.len();
        println!(
            "  Corpus: {} notes ({} en, {} ar, {} mixed)\n",
            note_count,
            EN_BODIES.len(),
            AR_BODIES.len(),
            MIXED_BODIES.len(),
        );

        const WARMUP: usize = 20;
        const SAMPLES: usize = 500;

        // ── Shape (a): known-word → bridges ──────────────────────────
        // These queries are in lexicon_v1.tsv so `expanded_match_query`
        // returns Some(...) with " OR "-joined cross-lingual branches.
        // Hit counts should span both source-language rows and bridged
        // rows in other scripts — which is the defining M12-wire win.
        println!("  ── Shape (a): known-word → bridges ──");
        let shape_a = &[
            ("tree (en→bridges)", "tree"),
            ("كتاب (ar→bridges)", "كتاب"),
            ("livre (fr→bridges)", "livre"),
        ];
        let mut worst_a_p99: u64 = 0;
        for (label, q) in shape_a {
            // One-shot hit count (measured separately so the per-call
            // measurement stays untainted by Vec<SearchResult> drop).
            let hits = lexical_search(&conn, q, 25).len();
            let s = measure(&conn, q, WARMUP, SAMPLES);
            report_stats(label, &s, hits);
            worst_a_p99 = worst_a_p99.max(percentile(&s, 0.99));
        }

        // ── Shape (b): unknown-word → prefix fallback ────────────────
        // `expanded_match_query` returns None (lemma not in corpus),
        // so `lexical_search` takes the raw `{word}*` prefix path —
        // identical shape to pre-M10 behaviour, included as the "null
        // hypothesis" baseline. If the bridged (a) path drifts far
        // from (b), we know the cost is in expansion/MATCH planning
        // rather than in FTS5 row retrieval.
        println!("\n  ── Shape (b): unknown-word → prefix fallback ──");
        let shape_b = &[
            ("quasar (prefix)", "quasar"),
            ("Constellation (prefix)", "Constellation"),
            ("xyzzy (prefix)", "xyzzy"),
        ];
        let mut worst_b_p99: u64 = 0;
        for (label, q) in shape_b {
            let hits = lexical_search(&conn, q, 25).len();
            let s = measure(&conn, q, WARMUP, SAMPLES);
            report_stats(label, &s, hits);
            worst_b_p99 = worst_b_p99.max(percentile(&s, 0.99));
        }

        // ── Shape (c): Arabic-only query (non-regression gate) ──────
        // The critical measurement: an Arabic query that IS in the
        // lexicon (so expansion fires, adding ~15 OR branches to the
        // FTS5 MATCH clause). Each added branch is a zero-hit lookup
        // against the Arabic-only rows that ultimately satisfy the
        // query. We're asserting that the added planning/scan cost
        // is negligible on FTS5 at this corpus scale — i.e. M12-wire
        // did not introduce a per-branch cost that scales with the
        // MATCH expression length.
        println!("\n  ── Shape (c): Arabic-only (non-regression gate) ──");
        let shape_c = &[
            ("شجرة (ar-only)", "شجرة"),
            ("معرفة (ar-only)", "معرفة"),
        ];
        let mut worst_c_p99: u64 = 0;
        for (label, q) in shape_c {
            let hits = lexical_search(&conn, q, 25).len();
            let s = measure(&conn, q, WARMUP, SAMPLES);
            report_stats(label, &s, hits);
            worst_c_p99 = worst_c_p99.max(percentile(&s, 0.99));
        }

        println!(
            "\n  Summary: worst-p99  (a)={:.2} ms  (b)={:.2} ms  (c)={:.2} ms    budget={:.0} ms",
            worst_a_p99 as f64 / 1_000_000.0,
            worst_b_p99 as f64 / 1_000_000.0,
            worst_c_p99 as f64 / 1_000_000.0,
            BUDGET_P99_NS as f64 / 1_000_000.0,
        );

        // Cleanup: bench is ephemeral, tempfiles don't survive.
        let _ = std::fs::remove_dir_all(&dir);

        // Hard gates. Order matters for diagnosis — if (c) trips first
        // the regression is in the bridged Arabic path specifically; if
        // (b) also trips the cost is elsewhere in FTS5 / rusqlite /
        // result materialisation. (a) is informational-hard (still
        // asserts budget so a bridged-path regression gets caught) but
        // (c) is the milestone-defining assertion.
        assert!(
            worst_a_p99 < BUDGET_P99_NS,
            "Shape (a) bridged p99 {} ns exceeds budget {} ns — bridged MATCH or FTS plan slowed down",
            worst_a_p99,
            BUDGET_P99_NS,
        );
        assert!(
            worst_b_p99 < BUDGET_P99_NS,
            "Shape (b) prefix p99 {} ns exceeds budget {} ns — baseline FTS5 path regressed (not M12-wire's fault)",
            worst_b_p99,
            BUDGET_P99_NS,
        );
        assert!(
            worst_c_p99 < BUDGET_P99_NS,
            "Shape (c) Arabic-only p99 {} ns exceeds budget {} ns — M12-wire HAS regressed Arabic search",
            worst_c_p99,
            BUDGET_P99_NS,
        );
    }
}
