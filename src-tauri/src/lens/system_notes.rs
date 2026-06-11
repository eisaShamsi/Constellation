//! MIG-055 §E — Five Acts system notes.
//!
//! Idempotent initializer for the system-shipped Five Acts host notes.
//! Called once at universe-init time (from `ensure_search_db_ready`,
//! after `init_db` completes). Per Architect §11 #3 lock — the
//! **transfer-on-edit** invariant: the system never overwrites a file
//! the user has edited.
//!
//! ## v1 host notes
//!
//! Just one note in v1: `{universe}/Five Acts/Observation — Recent Captures.md`.
//! Future phases extend the Five Acts set (Connection, Tension, Synthesis,
//! Conviction) — each is a new host note shipped from this module.
//!
//! ## Edit-policy invariant
//!
//! ```text
//! if file absent                  → create with canonical content
//! if file present (any content)   → no-op (transfer-on-edit honored)
//! ```
//!
//! The frontmatter marker `template: five-acts.observation` is the
//! lineage record. If the user edits the file, the system never
//! re-creates it on subsequent boots. If the user deletes it, the
//! next boot re-creates the canonical content (the "I want it back"
//! recovery path).

use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// Filename of the v1 Observation host note.
pub const RECENT_CAPTURES_FILENAME: &str = "Observation — Recent Captures.md";

/// Directory under the universe root that hosts the Five Acts notes.
pub const FIVE_ACTS_DIR: &str = "Five Acts";

/// Canonical content of the Observation — Recent Captures host note.
///
/// The fenced ` ```base ` block is what the §D LensBlock renderer
/// mounts on. The YAML inside the block is the v1.4 canonical Recent
/// Captures lens (Architect §6 fixture; tested in §C / §G).
///
/// Line endings are `\n` (Unix). The CM6 editor normalizes to `\n` on
/// save, so this matches what an unedited file looks like after the
/// user opens + closes it without changes.
pub const RECENT_CAPTURES_CONTENT: &str = r#"---
template: five-acts.observation
description: "The intake queue — last 14 days of notes. Browse what you've recently captured."
---

# Observation — Recent Captures

The Observation Act of knowledge formulation is **noticing**. Before connecting, tensing, synthesizing, or committing, you must first SEE what you've recently captured. This page shows you the last 14 days of notes across your universe — your intake queue.

Scan, read, mark as processed, or develop further. The list is your raw material.

```base
schema: 1
lens: "Recent Captures"
template: five-acts.observation
scope:
  libraries: all
  federation: auto
where:
  - dimension: note.created_at
    op: after
    value: "now - 14 days"
order:
  - dimension: note.created_at
    direction: desc
columns:
  - dimension: note.name
  - dimension: note.headline
view: list
```
"#;

/// Initialize the Five Acts system notes for the active universe.
///
/// Called once at boot from `crate::search::ensure_search_db_ready`.
/// Idempotent and edit-preserving — see module docstring for the
/// transfer-on-edit invariant.
///
/// Errors only on filesystem failures (permission denied / disk full /
/// invalid path); a non-existent universe directory means the function
/// is a no-op (caller handles universe-not-set as a separate case).
pub fn init_five_acts_system_notes(app: &AppHandle) -> Result<(), String> {
    let universe_dir = crate::universe::active_universe_dir(app)?;
    init_at(&universe_dir)
}

/// Implementation pulled out so tests can drive it without a Tauri app.
pub(crate) fn init_at(universe_root: &Path) -> Result<(), String> {
    let five_acts_dir = universe_root.join(FIVE_ACTS_DIR);

    // Create the Five Acts directory if it doesn't exist.
    // `create_dir_all` is idempotent — already-exists is not an error.
    fs::create_dir_all(&five_acts_dir).map_err(|e| {
        format!(
            "Failed to create `{}` directory at {}: {}",
            FIVE_ACTS_DIR,
            five_acts_dir.display(),
            e
        )
    })?;

    let target_path = five_acts_dir.join(RECENT_CAPTURES_FILENAME);

    if target_path.exists() {
        // Already exists — leave alone. The transfer-on-edit invariant
        // applies even when content == canonical, because we can't
        // distinguish "user opened it and saved without changes" from
        // "system created it and user never touched it". The conservative
        // choice (per Architect §11 #3 lock) is: NEVER overwrite an
        // existing file.
        return Ok(());
    }

    // MIG-076 §A2 — gated.
    crate::write_gate::gate_write(&target_path, RECENT_CAPTURES_CONTENT, None, "system_note")
        .map_err(|e| {
            format!(
                "Failed to write Five Acts system note {}: {}",
                target_path.display(),
                e
            )
        })?;

    Ok(())
}

/// Public Tauri-command DTO. Mirrors the `(display_name, relative_path)`
/// pair returned by `list_five_acts_notes_at`, but with field names the
/// frontend can read directly via serde. `absolute_path` is also returned
/// so the frontend opens the file with the same path resolver it uses
/// elsewhere (the universe-root prefix isn't needed by the open path).
#[derive(Debug, Clone, serde::Serialize)]
pub struct FiveActsNoteEntry {
    /// File stem (filename without `.md`), e.g., "Observation — Recent Captures".
    pub display_name: String,
    /// Universe-relative path, e.g., "Five Acts/Observation — Recent Captures.md".
    pub relative_path: String,
    /// Absolute filesystem path (resolves to the same file as `universe_root + relative_path`).
    pub absolute_path: String,
    /// MIG-062 — `None` for the active universe; `Some(name)` for a federated
    /// cUniverse. The sidebar groups entries by this into collapsible
    /// per-universe sub-groups. Read-only federation: the cUniverse's own
    /// Five Acts files are displayed, never moved/deleted (detach is lossless).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe_name: Option<String>,
}

/// MIG-055 §F — Tauri command. Frontend (`+layout.svelte` sidebar)
/// calls this to populate the "Five Acts" sidebar section. Returns
/// the canonical Observation — Recent Captures host note plus any
/// future Five Acts host notes that ship later (Connection, Tension,
/// Synthesis, Conviction).
#[tauri::command]
pub fn list_five_acts_notes(app: AppHandle) -> Result<Vec<FiveActsNoteEntry>, String> {
    let universe_dir = crate::universe::active_universe_dir(&app)?;
    // Active universe — universe_name = None.
    let mut out: Vec<FiveActsNoteEntry> = list_five_acts_notes_at(&universe_dir)?
        .into_iter()
        .map(|(display_name, rel)| {
            let absolute = universe_dir.join(&rel);
            FiveActsNoteEntry {
                display_name,
                relative_path: rel.to_string_lossy().replace('\\', "/"),
                absolute_path: absolute.to_string_lossy().to_string(),
                universe_name: None,
            }
        })
        .collect();

    // MIG-062 §C — federate READ-ONLY over the full cUniverse tree. Each
    // cUniverse's Five Acts notes are read and displayed; nothing is
    // written/moved/deleted. The cUniverse keeps its own files, so detaching
    // it leaves its Five Acts intact (Boss principle: "the wheel is already
    // there"). A per-cUniverse read failure is non-fatal (skip that one).
    for cu_root in crate::universe::resolve_child_universe_roots_recursive(&universe_dir) {
        let cu_name = crate::universe::universe_display_name(&cu_root);
        if let Ok(pairs) = list_five_acts_notes_at(&cu_root) {
            for (display_name, rel) in pairs {
                let absolute = cu_root.join(&rel);
                out.push(FiveActsNoteEntry {
                    display_name,
                    relative_path: rel.to_string_lossy().replace('\\', "/"),
                    absolute_path: absolute.to_string_lossy().to_string(),
                    universe_name: Some(cu_name.clone()),
                });
            }
        }
    }
    Ok(out)
}

/// Enumerate the `.md` files in `{universe}/Five Acts/` for the §F sidebar.
/// Returns `Vec<(display_name, relative_path)>` where `relative_path` is
/// relative to the universe root (so the frontend can open it via the same
/// path resolver it uses for any other note).
///
/// Empty vec if the directory doesn't exist. Non-fatal — the sidebar
/// renders an empty section gracefully.
pub fn list_five_acts_notes_at(universe_root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let five_acts_dir = universe_root.join(FIVE_ACTS_DIR);
    if !five_acts_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let entries = fs::read_dir(&five_acts_dir).map_err(|e| {
        format!(
            "Failed to list `{}` directory: {}",
            five_acts_dir.display(),
            e
        )
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        // Relative path = "Five Acts/<filename>"
        let rel = PathBuf::from(FIVE_ACTS_DIR).join(
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default(),
        );
        out.push((file_stem, rel));
    }
    // Stable order so the sidebar is reproducible.
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

// ─── §E tests ───

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_universe() -> TempDir {
        TempDir::new().expect("create tempdir")
    }

    #[test]
    fn fresh_universe_creates_canonical_file() {
        let universe = make_universe();
        init_at(universe.path()).expect("init_at OK on fresh universe");

        let expected_path = universe
            .path()
            .join(FIVE_ACTS_DIR)
            .join(RECENT_CAPTURES_FILENAME);
        assert!(
            expected_path.exists(),
            "system note should be created at {:?}",
            expected_path
        );

        let content = fs::read_to_string(&expected_path).unwrap();
        assert_eq!(
            content, RECENT_CAPTURES_CONTENT,
            "canonical content should be written verbatim"
        );
    }

    #[test]
    fn existing_canonical_file_left_unchanged() {
        let universe = make_universe();
        // First call creates.
        init_at(universe.path()).unwrap();
        // Second call must be a no-op (no overwrite, same mtime).
        let file_path = universe
            .path()
            .join(FIVE_ACTS_DIR)
            .join(RECENT_CAPTURES_FILENAME);
        let mtime_before = fs::metadata(&file_path).unwrap().modified().unwrap();
        // Sleep ~10ms to make the mtime check meaningful — on Windows
        // filesystems with low-resolution timestamps this still works
        // because the file simply isn't written, so mtime is unchanged.
        std::thread::sleep(std::time::Duration::from_millis(10));
        init_at(universe.path()).unwrap();
        let mtime_after = fs::metadata(&file_path).unwrap().modified().unwrap();
        assert_eq!(
            mtime_before, mtime_after,
            "second init_at should not touch the file"
        );
    }

    #[test]
    fn existing_user_edited_file_left_unchanged() {
        let universe = make_universe();
        let five_acts = universe.path().join(FIVE_ACTS_DIR);
        fs::create_dir_all(&five_acts).unwrap();
        let file_path = five_acts.join(RECENT_CAPTURES_FILENAME);
        let user_content = "# My custom Observation\n\nUser content here.";
        fs::write(&file_path, user_content).unwrap();

        init_at(universe.path()).unwrap();

        let after = fs::read_to_string(&file_path).unwrap();
        assert_eq!(
            after, user_content,
            "user-edited file MUST NOT be overwritten (transfer-on-edit)"
        );
    }

    #[test]
    fn missing_five_acts_directory_is_created() {
        let universe = make_universe();
        let five_acts_dir = universe.path().join(FIVE_ACTS_DIR);
        assert!(!five_acts_dir.exists(), "precondition: dir absent");

        init_at(universe.path()).unwrap();

        assert!(five_acts_dir.is_dir(), "directory should now exist");
    }

    #[test]
    fn two_consecutive_inits_are_idempotent() {
        let universe = make_universe();
        init_at(universe.path()).unwrap();
        init_at(universe.path()).unwrap();
        init_at(universe.path()).unwrap();

        let file_path = universe
            .path()
            .join(FIVE_ACTS_DIR)
            .join(RECENT_CAPTURES_FILENAME);
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, RECENT_CAPTURES_CONTENT);

        // Directory contains exactly one .md file.
        let entries: Vec<_> = fs::read_dir(universe.path().join(FIVE_ACTS_DIR))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    == Some("md")
            })
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "no duplicates after three consecutive inits"
        );
    }

    #[test]
    fn canonical_content_contains_recent_captures_lens_yaml() {
        // Sanity: the embedded YAML matches what §C's parser tests expect.
        assert!(RECENT_CAPTURES_CONTENT.contains("```base"));
        assert!(RECENT_CAPTURES_CONTENT.contains("schema: 1"));
        assert!(RECENT_CAPTURES_CONTENT.contains("lens: \"Recent Captures\""));
        assert!(RECENT_CAPTURES_CONTENT.contains("template: five-acts.observation"));
        assert!(RECENT_CAPTURES_CONTENT.contains("now - 14 days"));
        assert!(RECENT_CAPTURES_CONTENT.contains("note.created_at"));
        assert!(RECENT_CAPTURES_CONTENT.contains("note.name"));
        assert!(RECENT_CAPTURES_CONTENT.contains("note.headline"));
    }

    #[test]
    fn canonical_yaml_round_trips_through_parser() {
        // Extract the YAML between ```base and ```, then parse it.
        let content = RECENT_CAPTURES_CONTENT;
        let start = content.find("```base\n").expect("```base fence present") + "```base\n".len();
        let end = content[start..]
            .find("```")
            .expect("closing ``` present");
        let yaml = &content[start..start + end];

        let def = crate::lens::parser::parse_lens_yaml(yaml)
            .expect("canonical lens YAML must parse");
        crate::lens::validator::validate(&def)
            .expect("canonical lens YAML must validate against §A registry");
        assert_eq!(def.lens, "Recent Captures");
        assert_eq!(def.schema, 1);
    }

    #[test]
    fn list_five_acts_notes_returns_empty_when_dir_absent() {
        let universe = make_universe();
        let listed = list_five_acts_notes_at(universe.path()).unwrap();
        assert_eq!(listed.len(), 0);
    }

    #[test]
    fn list_five_acts_notes_returns_canonical_after_init() {
        let universe = make_universe();
        init_at(universe.path()).unwrap();
        let listed = list_five_acts_notes_at(universe.path()).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].0, "Observation — Recent Captures");
        assert_eq!(
            listed[0].1,
            PathBuf::from(FIVE_ACTS_DIR).join(RECENT_CAPTURES_FILENAME)
        );
    }

    #[test]
    fn list_five_acts_notes_ignores_non_markdown_files() {
        let universe = make_universe();
        let five_acts = universe.path().join(FIVE_ACTS_DIR);
        fs::create_dir_all(&five_acts).unwrap();
        fs::write(five_acts.join("real.md"), "# real").unwrap();
        fs::write(five_acts.join("readme.txt"), "ignore me").unwrap();
        fs::write(five_acts.join("noteX.markdown"), "ignore me").unwrap();
        let listed = list_five_acts_notes_at(universe.path()).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].0, "real");
    }
}
