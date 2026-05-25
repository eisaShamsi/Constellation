//! Citation validator — every `[note:<path>]` reference in the
//! assistant's response must resolve to a real note in `note_meta`.
//! If any don't, the orchestrator re-prompts the model with feedback
//! (one retry per turn, per Eisa-locked decision C1 in MIG-048
//! Architect §9).
//!
//! ## Citation shape
//!
//! Phase 1 v1: `[note:<file_path>]` where `file_path` is the absolute
//! path the model received from a tool result (e.g., `search_notes`'s
//! `path` field). The validator performs `SELECT path FROM note_meta
//! WHERE path = ?` to confirm the path exists.
//!
//! ## Why path (not UUID)
//!
//! Paths are stable across content edits (more common than renames in
//! a PKM workflow) and the model already sees paths in tool results.
//! `cid_cn` would invalidate after every edit, making citations
//! brittle. Phase 1.x may add UUID resolution as an alternate citation
//! shape (`[note:<uuid>]`) once we mint per-note UUIDs.
//!
//! ## Retry budget
//!
//! Per Architect §C1: ONE retry per turn. The retry budget is consumed
//! independent of the MA-4 tool-call budget — they serve different
//! purposes and aren't summed.

use std::sync::OnceLock;

use regex::Regex;
use rusqlite::params;
use tauri::Manager;

use crate::search::SearchState;

/// Capture group: anything between `[note:` and `]` that isn't `]`.
static CITATION_RE: OnceLock<Regex> = OnceLock::new();

fn citation_re() -> &'static Regex {
    CITATION_RE.get_or_init(|| Regex::new(r"\[note:([^\]]+)\]").expect("static regex"))
}

/// Extract every `[note:X]` reference from `text`. Order preserved;
/// duplicates retained (so a caller can count "how many references"
/// rather than unique IDs).
pub fn scan_citations(text: &str) -> Vec<String> {
    citation_re()
        .captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Verification outcome for one path lookup. Distinguishes "the DB
/// confirmed the path is missing" (Missing) from "the DB couldn't be
/// queried" (Unverifiable). MIG-048 §M audit flagged the P1 risk that
/// a fail-CLOSED policy would mark every citation invalid on a fresh
/// install where the SearchState DB hasn't been opened yet.
#[derive(Debug, PartialEq, Eq)]
enum PathVerdict {
    Exists,
    Missing,
    Unverifiable,
}

fn verify_path(app: &tauri::AppHandle, path: &str) -> PathVerdict {
    let state = app.state::<SearchState>();
    let db_guard = match state.db.lock() {
        Ok(g) => g,
        Err(_) => return PathVerdict::Unverifiable,
    };
    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => return PathVerdict::Unverifiable,
    };
    match conn.query_row(
        "SELECT 1 FROM note_meta WHERE path = ?1 LIMIT 1",
        params![path],
        |_row| Ok(()),
    ) {
        Ok(()) => PathVerdict::Exists,
        Err(rusqlite::Error::QueryReturnedNoRows) => PathVerdict::Missing,
        // Any other SQL error → treat as unverifiable, not missing.
        Err(_) => PathVerdict::Unverifiable,
    }
}

/// Public wrapper retained for callers that only need a boolean
/// answer (tests, future callers). The boolean conflates Missing +
/// Unverifiable but most callers only care whether the path is real.
pub fn note_path_exists(app: &tauri::AppHandle, path: &str) -> bool {
    matches!(verify_path(app, path), PathVerdict::Exists)
}

/// Validate every `[note:X]` reference in `text`. Returns
/// `(valid_paths, invalid_paths)` (both deduplicated).
///
/// Fail-OPEN policy (§M P1 caveat fix): when the search DB is
/// unavailable (uninitialized / lock poisoned / SQL error other than
/// "no rows"), the path is treated as `valid` rather than `invalid`.
/// This avoids the failure mode where the validator marks EVERY
/// citation invalid on a fresh install and the user sees the warning
/// prefix constantly. Validation only catches REAL fabrications when
/// the DB CAN confirm the path is missing.
pub fn scan_and_verify(app: &tauri::AppHandle, text: &str) -> (Vec<String>, Vec<String>) {
    let mut valid: Vec<String> = Vec::new();
    let mut invalid: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for path in scan_citations(text) {
        if !seen.insert(path.clone()) {
            continue;
        }
        match verify_path(app, &path) {
            PathVerdict::Exists => valid.push(path),
            PathVerdict::Missing => invalid.push(path),
            // Fail open — treat as valid when verification is not possible.
            PathVerdict::Unverifiable => valid.push(path),
        }
    }

    (valid, invalid)
}

/// Compose the feedback message the orchestrator appends to history
/// when it detects invalid citations. The model sees this as a System
/// turn before being asked to regenerate.
pub fn feedback_message(invalid_paths: &[String]) -> String {
    let formatted = invalid_paths
        .iter()
        .map(|p| format!("- [note:{p}]"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Your previous response referenced these notes that don't exist in the user's Universe:\n\
         {formatted}\n\n\
         Either remove those claims, or replace them with citations to notes from the tool \
         results above. Do not invent paths. Respond again now."
    )
}

/// Compose the warning prefix the orchestrator prepends to the
/// assistant's text when the retry didn't fix the citations.
pub fn warning_prefix(invalid_paths: &[String]) -> String {
    let count = invalid_paths.len();
    let plural = if count == 1 { "citation" } else { "citations" };
    format!(
        "⚠ This response contains {count} unresolved {plural} ({first}). \
         Verify before trusting.\n\n",
        first = invalid_paths.first().map(String::as_str).unwrap_or("?"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_finds_simple_citation() {
        let t = "Canopus is bright [note:/lib/canopus.md].";
        let uuids = scan_citations(t);
        assert_eq!(uuids, vec!["/lib/canopus.md"]);
    }

    #[test]
    fn scan_finds_multiple_distinct_citations() {
        let t = "See [note:/a.md] and [note:/b.md]. Also [note:/a.md] again.";
        let uuids = scan_citations(t);
        assert_eq!(uuids, vec!["/a.md", "/b.md", "/a.md"]);
    }

    #[test]
    fn scan_handles_paths_with_spaces_and_unicode() {
        let t = "See [note:/مكتبة/سهيل.md] and [note:/My Notes/file.md].";
        let uuids = scan_citations(t);
        assert_eq!(uuids, vec!["/مكتبة/سهيل.md", "/My Notes/file.md"]);
    }

    #[test]
    fn scan_returns_empty_on_no_citations() {
        let t = "No citations here, just prose.";
        assert!(scan_citations(t).is_empty());
    }

    #[test]
    fn feedback_message_lists_each_invalid_path() {
        let invalid = vec!["/a.md".to_string(), "/b.md".to_string()];
        let msg = feedback_message(&invalid);
        assert!(msg.contains("[note:/a.md]"));
        assert!(msg.contains("[note:/b.md]"));
        assert!(msg.contains("don't exist"));
        assert!(msg.contains("Respond again now"));
    }

    #[test]
    fn warning_prefix_uses_singular_or_plural() {
        let one = vec!["/a.md".to_string()];
        let two = vec!["/a.md".to_string(), "/b.md".to_string()];
        assert!(warning_prefix(&one).contains("1 unresolved citation "));
        assert!(warning_prefix(&two).contains("2 unresolved citations "));
    }
}
