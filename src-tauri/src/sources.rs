//! MIG-021 §1A — Sources subsystem foundation.
//!
//! This module is the data substrate for Constellation Sight v5's
//! Provenance dimension (mode P). It defines:
//!
//! 1. The canonical 11-source vocabulary from the Universal Epistemic
//!    Content Taxonomy (`docs/epistemic-content-taxonomy.md`) +
//!    a 12th `unclassifiable` opt-out token (per MIG-021 Plan §0 Q5).
//! 2. The schema migration adding `note_meta.sources` column and
//!    `sources_suggestions` table.
//! 3. The frontmatter parser for `sources:` (handles all three YAML
//!    shapes: scalar, inline array, block list — same pattern as
//!    `search::extract_aliases`).
//! 4. Read/write helpers for both `sources:` (canonical, user-controlled)
//!    and `sources_suggestions` (transient classifier output, consumed
//!    on user approval).
//! 5. A frontmatter rewriter that updates the `sources:` block on disk
//!    while preserving every other frontmatter field and body content.
//! 6. Three `#[tauri::command]` IPCs surfaced to the frontend
//!    (`sources_get_for_note`, `sources_set_manual`, `sources_clear`).
//!
//! The classifier itself (Tier 1 e5-small embedding-similarity, Tier 2
//! Qwen3-1.7B via llama.cpp) is built in MIG-021 §1B in a sibling
//! `classifier` module — this file is foundation only.
//!
//! Anchored against:
//!   docs/Constellation-Sight-Concept-Paper-v2.0.md §7
//!   lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md §1A

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ─── Constants ─────────────────────────────────────────────────────

/// Schema version for the sources subsystem. Bumped when any
/// structural change to `note_meta.sources` or `sources_suggestions`
/// requires migration. v1 = initial introduction (MIG-021 §1A).
pub const SOURCES_SCHEMA_VERSION: i64 = 1;

/// The 11 canonical sources from the Universal Epistemic Content Taxonomy
/// + the 12th `unclassifiable` opt-out token (MIG-021 Plan §0 Q5).
///
/// Order is canonical; the frontend's PropertyEditor combobox should
/// render in this order (the Concept Paper §7.1 lists them in this
/// order, drawn from the taxonomy doc).
///
/// `unclassifiable` is a user-set opt-out that suppresses future
/// classifier suggestions on a note. It is NOT a real epistemic source;
/// the classifier never suggests it; Sight v5 mode P treats notes
/// tagged `unclassifiable` as a separate "user opted out" wedge,
/// distinct from the "Unsourced" wedge for notes with no `sources:` field.
pub const SOURCE_IDS: &[&str; 12] = &[
    "perception",
    "inference",
    "testimony",
    "mass-transmission",
    "comparison",
    "postulation",
    "non-apprehension",
    "memory",
    "innate-disposition",
    "inspiration",
    "revelation",
    "unclassifiable",
];

/// The 11 canonical sources only (excludes `unclassifiable`). Used by
/// the classifier to bound its suggestions to real epistemic sources.
pub fn classifiable_sources() -> &'static [&'static str] {
    &SOURCE_IDS[0..11]
}

// ─── Types ──────────────────────────────────────────────────────────

/// One classifier suggestion record for a single source candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub source: String,
    pub confidence: f32,
    pub evidence: String,
}

/// A complete suggestion record persisted in `sources_suggestions`
/// table and read by the Source Review panel (MIG-021 §1C).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionRecord {
    pub note_path: String,
    pub suggestions: Vec<Suggestion>,
    pub classifier_tier: i64, // 1 = embedding (Tier 1), 2 = LLM (Tier 2)
    pub created_at: i64,
}

// ─── Schema migration ──────────────────────────────────────────────

/// Idempotent schema migration adding the `sources` column to
/// `note_meta` if missing. Mirrors `search::ensure_note_meta_mig002_columns`
/// — SQLite lacks `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`, so we
/// probe `PRAGMA table_info` first.
///
/// Called from `search::init_db` after the existing MIG-002/003
/// column-ensures.
pub fn ensure_note_meta_sources_column(conn: &Connection) -> rusqlite::Result<()> {
    let mut have = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(note_meta)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for col in rows {
            if col? == "sources" {
                have = true;
                break;
            }
        }
    }
    if !have {
        conn.execute_batch(
            "ALTER TABLE note_meta ADD COLUMN sources TEXT DEFAULT NULL;",
        )?;
    }
    Ok(())
}

/// Create the `sources_suggestions` table for the classifier review queue
/// + its `created_at` index. Idempotent via `CREATE TABLE IF NOT EXISTS`.
///
/// `note_path` is the primary key (one suggestion record per note at any
/// given time; re-suggesting REPLACES the prior record). The foreign
/// key + cascade-delete keeps the queue clean when notes are deleted.
pub fn ensure_sources_suggestions_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sources_suggestions (
            note_path TEXT PRIMARY KEY,
            suggestions_json TEXT NOT NULL,
            classifier_tier INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (note_path) REFERENCES note_meta(path) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_sources_suggestions_created
            ON sources_suggestions(created_at);",
    )?;
    Ok(())
}

// ─── Frontmatter parsing ───────────────────────────────────────────

/// Extract YAML `sources:` from a note's frontmatter.
///
/// Mirrors `search::extract_aliases` (MIG-004 §2) — handles all three
/// YAML shapes Constellation accepts:
///
/// ```yaml
/// sources: testimony                        # scalar
/// sources: [testimony, inference]           # inline array
/// sources:                                  # block list
///   - testimony
///   - inference
/// ```
///
/// Returns the ordered list (primary first). Unknown values that don't
/// match `SOURCE_IDS` are silently dropped — the user's frontmatter
/// is canonical, but we don't propagate typos into the SQLite mirror.
///
/// Block-aware: tracks "are we inside the `sources:` block" so a `-`
/// line item that follows `tags:` or another list field is NOT
/// mistakenly consumed.
pub fn extract_sources(content: &str) -> Vec<String> {
    if !content.starts_with("---") {
        return Vec::new();
    }
    let Some(end) = content[3..].find("\n---") else {
        return Vec::new();
    };
    let frontmatter = &content[3..3 + end];

    let mut out: Vec<String> = Vec::new();
    let mut in_block = false;

    for line in frontmatter.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("sources:") {
            in_block = true;
            let value = trimmed["sources:".len()..].trim();
            if value.starts_with('[') && value.ends_with(']') {
                // Inline array.
                let inner = &value[1..value.len() - 1];
                for raw in inner.split(',') {
                    push_source(&mut out, raw);
                }
                in_block = false;
            } else if !value.is_empty() {
                // Scalar.
                push_source(&mut out, value);
                in_block = false;
            }
            // else: block list — items consumed below.
            continue;
        }

        if in_block {
            if let Some(rest) = trimmed.strip_prefix("- ") {
                push_source(&mut out, rest);
                continue;
            }
            // Any non-list-item line ends the block (next field, etc.).
            if !trimmed.is_empty() {
                in_block = false;
            }
        }
    }
    out
}

fn push_source(out: &mut Vec<String>, raw: &str) {
    let normalized = raw
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_lowercase();
    if normalized.is_empty() {
        return;
    }
    if !SOURCE_IDS.iter().any(|s| *s == normalized) {
        return; // Silent drop of unknown values.
    }
    if !out.contains(&normalized) {
        out.push(normalized);
    }
}

// ─── DB read/write ─────────────────────────────────────────────────

/// Read the `sources` JSON list from `note_meta` for a given note.
/// Returns an empty Vec if the note is unknown, the column is NULL,
/// or the JSON fails to parse (with a warning logged).
pub fn read_sources_for_note(conn: &Connection, note_path: &str) -> Result<Vec<String>, String> {
    let json: Option<String> = conn
        .query_row(
            "SELECT sources FROM note_meta WHERE path = ?1",
            params![note_path],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    match json {
        None => Ok(Vec::new()),
        Some(s) if s.is_empty() => Ok(Vec::new()),
        Some(s) => serde_json::from_str(&s)
            .map_err(|e| format!("Failed to parse sources JSON for {}: {}", note_path, e)),
    }
}

/// Write a sources list to `note_meta.sources` (validated against
/// `SOURCE_IDS`; unknown values are dropped). Caller is responsible
/// for ensuring the note row exists in `note_meta` first.
pub fn write_sources_to_db(
    conn: &Connection,
    note_path: &str,
    sources: &[String],
) -> Result<(), String> {
    let validated: Vec<&str> = sources
        .iter()
        .filter_map(|s| {
            SOURCE_IDS
                .iter()
                .find(|id| **id == s.as_str())
                .copied()
        })
        .collect();
    let json = serde_json::to_string(&validated)
        .map_err(|e| format!("Failed to serialize sources: {}", e))?;
    conn.execute(
        "UPDATE note_meta SET sources = ?1 WHERE path = ?2",
        params![json, note_path],
    )
    .map_err(|e| format!("Failed to update note_meta.sources for {}: {}", note_path, e))?;
    Ok(())
}

/// Read the suggestion record for a note from `sources_suggestions`.
/// Returns `None` if no suggestion is queued for the note.
pub fn read_suggestions(
    conn: &Connection,
    note_path: &str,
) -> Result<Option<SuggestionRecord>, String> {
    let row: Option<(String, i64, i64)> = conn
        .query_row(
            "SELECT suggestions_json, classifier_tier, created_at
             FROM sources_suggestions WHERE note_path = ?1",
            params![note_path],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();
    match row {
        None => Ok(None),
        Some((json, tier, created)) => {
            let suggestions: Vec<Suggestion> = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse suggestions for {}: {}", note_path, e))?;
            Ok(Some(SuggestionRecord {
                note_path: note_path.to_string(),
                suggestions,
                classifier_tier: tier,
                created_at: created,
            }))
        }
    }
}

/// Write (or replace) a suggestion record. Replaces existing entry
/// for the note since a note can only have one pending suggestion at
/// a time.
pub fn write_suggestions(
    conn: &Connection,
    note_path: &str,
    suggestions: &[Suggestion],
    tier: i64,
) -> Result<(), String> {
    let json = serde_json::to_string(suggestions)
        .map_err(|e| format!("Failed to serialize suggestions: {}", e))?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO sources_suggestions
         (note_path, suggestions_json, classifier_tier, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![note_path, json, tier, now],
    )
    .map_err(|e| format!("Failed to write suggestion for {}: {}", note_path, e))?;
    Ok(())
}

/// Clear the suggestion record for a note (consumed on Accept or Reject).
pub fn clear_suggestions(conn: &Connection, note_path: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM sources_suggestions WHERE note_path = ?1",
        params![note_path],
    )
    .map_err(|e| format!("Failed to clear suggestion for {}: {}", note_path, e))?;
    Ok(())
}

// ─── Frontmatter writer ────────────────────────────────────────────

/// Rewrite a note's frontmatter to update the `sources:` field.
///
/// - If the note has no frontmatter and `sources` is non-empty,
///   prepends a new frontmatter block.
/// - If the note has frontmatter, removes any existing `sources:`
///   field (scalar / inline / block) and appends the new list as a
///   block-style YAML list. Other frontmatter fields and body content
///   are preserved verbatim.
/// - If `sources` is empty, the field is removed entirely (the note
///   becomes "unsourced" again).
///
/// Returns the rewritten string. Caller writes to disk.
pub fn rewrite_frontmatter_sources(content: &str, sources: &[String]) -> String {
    let validated: Vec<&str> = sources
        .iter()
        .filter_map(|s| {
            SOURCE_IDS
                .iter()
                .find(|id| **id == s.as_str())
                .copied()
        })
        .collect();

    if !content.starts_with("---") {
        if validated.is_empty() {
            return content.to_string();
        }
        // Synthesize a minimal frontmatter block.
        let mut out = String::from("---\nsources:\n");
        for s in &validated {
            out.push_str("  - ");
            out.push_str(s);
            out.push('\n');
        }
        out.push_str("---\n\n");
        out.push_str(content);
        return out;
    }

    let Some(end) = content[3..].find("\n---") else {
        // Malformed frontmatter — leave alone.
        return content.to_string();
    };
    let fm = &content[3..3 + end];
    let body_start = 3 + end + 4; // skip past "\n---" closing delimiter
    let body = &content[body_start..];

    // Strip any existing `sources:` block (scalar / inline / block list).
    let mut new_fm_lines: Vec<String> = Vec::new();
    let mut skip_block = false;
    for line in fm.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("sources:") {
            // Determine if scalar/inline (single line) vs block (multi-line).
            let value = trimmed["sources:".len()..].trim();
            if value.is_empty() {
                // Block list — skip following `- ` lines.
                skip_block = true;
            }
            continue; // drop this line either way
        }
        if skip_block {
            if trimmed.starts_with("- ") {
                continue; // still inside the dropped block
            } else if !trimmed.is_empty() {
                skip_block = false;
                // fall through to push this line
            } else {
                continue; // blank inside block — drop
            }
        }
        new_fm_lines.push(line.to_string());
    }

    let mut new_fm = new_fm_lines.join("\n");
    while new_fm.ends_with('\n') || new_fm.ends_with(' ') {
        new_fm.pop();
    }

    if !validated.is_empty() {
        new_fm.push_str("\nsources:\n");
        for s in &validated {
            new_fm.push_str("  - ");
            new_fm.push_str(s);
            new_fm.push('\n');
        }
    } else {
        new_fm.push('\n');
    }

    let mut out = String::from("---");
    out.push_str(&new_fm);
    out.push_str("---");
    out.push_str(body);
    out
}

// ─── Disk I/O for IPC ─────────────────────────────────────────────

/// Helper used by the manual-set IPC: read a note's content from disk,
/// rewrite the `sources:` frontmatter, write back. Atomic via a
/// temp-file rename in the `tempfile` style of MIG-006 §9 (deferred
/// — for now uses direct write; concurrent writes during user-set
/// are extremely unlikely given the UI flow).
fn rewrite_note_sources_on_disk(note_path: &str, sources: &[String]) -> Result<(), String> {
    let path = Path::new(note_path);
    if !path.exists() {
        return Err(format!("Note not found: {}", note_path));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", note_path, e))?;
    let rewritten = rewrite_frontmatter_sources(&content, sources);
    std::fs::write(path, rewritten)
        .map_err(|e| format!("Failed to write {}: {}", note_path, e))?;
    Ok(())
}

// ─── Tauri commands ────────────────────────────────────────────────

/// Read the canonical `sources:` list for a single note from the
/// `note_meta` SQLite mirror. Returns empty list if note is unsourced.
#[tauri::command]
pub fn sources_get_for_note(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<Vec<String>, String> {
    crate::search::ensure_search_db_ready(&app)?;
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    read_sources_for_note(conn, &note_path)
}

/// Manually set the canonical `sources:` list for a note. Writes both
/// the frontmatter on disk AND the `note_meta.sources` mirror.
/// Frontmatter wins on disagreement; the mirror is rebuilt from
/// frontmatter on the next `index_note` pass.
///
/// Also clears any pending classifier suggestion for the note (the
/// user has spoken; no need to surface a suggestion they'll override).
#[tauri::command]
pub fn sources_set_manual(
    app: tauri::AppHandle,
    note_path: String,
    sources: Vec<String>,
) -> Result<(), String> {
    crate::search::ensure_search_db_ready(&app)?;
    // Validate all values are in SOURCE_IDS up front; reject the call
    // entirely if any unknown source slipped through (the frontend
    // should never send unknowns, but defense-in-depth).
    for s in &sources {
        if !SOURCE_IDS.iter().any(|id| *id == s.as_str()) {
            return Err(format!("Unknown source ID: {}", s));
        }
    }

    // 1. Write frontmatter to disk (canonical store).
    rewrite_note_sources_on_disk(&note_path, &sources)?;

    // 2. Update note_meta.sources mirror.
    {
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard
            .as_ref()
            .ok_or("Search database not initialized")?;
        write_sources_to_db(conn, &note_path, &sources)?;
        // 3. Clear any pending suggestion (consumed by user action).
        clear_suggestions(conn, &note_path)?;
    }

    Ok(())
}

/// Clear the `sources:` field for a note (returns it to "unsourced"
/// state). Removes both from frontmatter and from the `note_meta`
/// mirror. Does NOT clear classifier suggestions — the next scan
/// can re-propose.
#[tauri::command]
pub fn sources_clear(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<(), String> {
    crate::search::ensure_search_db_ready(&app)?;
    let empty: Vec<String> = Vec::new();

    rewrite_note_sources_on_disk(&note_path, &empty)?;

    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    write_sources_to_db(conn, &note_path, &empty)?;
    Ok(())
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_sources_handles_scalar() {
        let content = "---\ntitle: Foo\nsources: testimony\n---\n\nbody";
        assert_eq!(extract_sources(content), vec!["testimony"]);
    }

    #[test]
    fn extract_sources_handles_inline_array() {
        let content = "---\nsources: [testimony, inference]\n---\nbody";
        assert_eq!(
            extract_sources(content),
            vec!["testimony".to_string(), "inference".to_string()]
        );
    }

    #[test]
    fn extract_sources_handles_block_list() {
        let content = "---\nsources:\n  - testimony\n  - mass-transmission\n---\nbody";
        assert_eq!(
            extract_sources(content),
            vec![
                "testimony".to_string(),
                "mass-transmission".to_string()
            ]
        );
    }

    #[test]
    fn extract_sources_drops_unknown() {
        let content = "---\nsources: [testimony, fake-source, inference]\n---\nbody";
        assert_eq!(
            extract_sources(content),
            vec!["testimony".to_string(), "inference".to_string()]
        );
    }

    #[test]
    fn extract_sources_no_frontmatter() {
        assert_eq!(extract_sources("just body").len(), 0);
    }

    #[test]
    fn extract_sources_preserves_order() {
        let content = "---\nsources:\n  - inference\n  - testimony\n  - revelation\n---";
        assert_eq!(
            extract_sources(content),
            vec![
                "inference".to_string(),
                "testimony".to_string(),
                "revelation".to_string()
            ]
        );
    }

    #[test]
    fn extract_sources_block_terminates_on_other_field() {
        let content = "---\nsources:\n  - testimony\ntags: [foo]\n  - bar\n---";
        // The `- bar` line should NOT be consumed — block ended at `tags:`.
        assert_eq!(extract_sources(content), vec!["testimony".to_string()]);
    }

    #[test]
    fn rewrite_inserts_into_existing_frontmatter() {
        let content = "---\ntitle: Foo\n---\n\nbody";
        let rewritten = rewrite_frontmatter_sources(&content.to_string(), &["testimony".to_string()]);
        assert!(rewritten.contains("sources:"));
        assert!(rewritten.contains("- testimony"));
        assert!(rewritten.contains("title: Foo"));
        assert!(rewritten.contains("body"));
    }

    #[test]
    fn rewrite_replaces_existing_sources() {
        let content = "---\ntitle: Foo\nsources:\n  - inference\n---\n\nbody";
        let rewritten =
            rewrite_frontmatter_sources(&content.to_string(), &["testimony".to_string()]);
        assert!(rewritten.contains("- testimony"));
        assert!(!rewritten.contains("- inference"));
        assert!(rewritten.contains("title: Foo"));
        assert!(rewritten.contains("body"));
    }

    #[test]
    fn rewrite_clears_when_empty() {
        let content = "---\ntitle: Foo\nsources:\n  - inference\n---\n\nbody";
        let rewritten = rewrite_frontmatter_sources(&content.to_string(), &[]);
        assert!(!rewritten.contains("sources:"));
        assert!(rewritten.contains("title: Foo"));
        assert!(rewritten.contains("body"));
    }

    #[test]
    fn rewrite_synthesizes_frontmatter_when_missing() {
        let content = "just body";
        let rewritten = rewrite_frontmatter_sources(&content.to_string(), &["testimony".to_string()]);
        assert!(rewritten.starts_with("---"));
        assert!(rewritten.contains("sources:"));
        assert!(rewritten.contains("- testimony"));
        assert!(rewritten.contains("just body"));
    }

    #[test]
    fn unclassifiable_is_canonical() {
        assert!(SOURCE_IDS.contains(&"unclassifiable"));
        assert_eq!(classifiable_sources().len(), 11);
        assert!(!classifiable_sources().contains(&"unclassifiable"));
    }
}
