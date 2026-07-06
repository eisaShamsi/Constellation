use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
// MIG-065 §I-b — base CREATION now writes a minimal LensDefinition YAML (the
// unified engine's format), not the old BaseDefinition JSON.
use crate::lens::definition::{
    FederationMode, LensColumn, LensDefinition, LensFilter, LensScope, LensSort, LensView,
    LibrariesSelector, SortDirection,
};
// tauri::Manager unused — removed

// ─── Security ───

/// Validate that a file path is within a registered library or the active universe's bases directory.
/// MIG-065 §G — `pub(crate)` so the unified lens engine's `update_base_columns`
/// reuses the same universe/library scoping when it rewrites a `.base` file.
pub(crate) fn validate_base_path(app: &tauri::AppHandle, file_path: &str) -> Result<(), String> {
    let target = fs::canonicalize(file_path)
        .or_else(|_| {
            // File may not exist yet (save); canonicalize parent
            Path::new(file_path).parent()
                .ok_or_else(|| "Invalid path".to_string())
                .and_then(|p| fs::canonicalize(p).map_err(|e| e.to_string()))
        })
        .map_err(|_| "Cannot resolve file path.".to_string())?;

    // Check if path is within the active universe directory
    if let Ok(universe_dir) = crate::universe::active_universe_dir(app) {
        if let Ok(canon_universe) = fs::canonicalize(&universe_dir) {
            if target.starts_with(&canon_universe) {
                return Ok(());
            }
        }
    }

    // Check if path is within one of the ACTIVE universe's OWN libraries
    // (non-recursive — MIG-065 §J: a write must never be authorized onto a
    // read-only cUniverse `.base`; the recursive set would include it).
    let libraries = crate::libraries::load_libraries(app);
    for lib in &libraries {
        if let Ok(canon_lib) = fs::canonicalize(&lib.path) {
            if target.starts_with(&canon_lib) {
                return Ok(());
            }
        }
    }

    Err("Path is outside of registered libraries and universe directory.".to_string())
}

// ─── Data Structures ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSource {
    #[serde(rename = "type")]
    pub source_type: String,   // "folder" | "tag" | "all"
    pub path: Option<String>,  // folder path (relative to library root)
    pub tag: Option<String>,   // tag filter
    #[serde(rename = "includeSubfolders", default = "default_true")]
    pub include_subfolders: bool,
    #[serde(rename = "selectedVaults", default)]
    pub selected_vaults: Vec<String>, // empty = all libraries; populated = only these library names
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub property: String,
    pub label: Option<String>,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub direction: Option<String>, // per-column direction override
}

fn default_width() -> u32 { 150 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    pub property: String,
    pub operator: String, // "is" | "is_not" | "contains" | "not_contains" | "gt" | "lt" | "is_empty" | "is_not_empty"
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortRule {
    pub property: String,
    #[serde(default = "default_asc")]
    pub direction: String, // "asc" | "desc"
}

fn default_asc() -> String { "asc".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDefinition {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub name: String,
    pub source: BaseSource,
    #[serde(default)]
    pub columns: Vec<ColumnDef>,
    #[serde(default)]
    pub filters: Vec<FilterRule>,
    #[serde(default)]
    pub sorts: Vec<SortRule>,
    #[serde(default = "default_view")]
    pub view: String, // "table" | "card" | "list"
    #[serde(default = "default_auto")]
    pub direction: String, // "auto" | "rtl" | "ltr"
}

fn default_version() -> u32 { 1 }
fn default_view() -> String { "table".to_string() }
fn default_auto() -> String { "auto".to_string() }

// ─── Frontmatter Parser ───

/// Parse YAML frontmatter from a markdown note into a HashMap.
/// Returns None if no valid frontmatter found.
pub fn parse_frontmatter(content: &str) -> Option<HashMap<String, String>> {
    if !content.starts_with("---") {
        return None;
    }
    let lines: Vec<&str> = content.lines().collect();
    let end_idx = lines.iter().skip(1).position(|l| l.trim() == "---")?;
    let end_idx = end_idx + 1; // offset from skip(1)

    let mut props = HashMap::new();
    let mut i = 1;
    while i < end_idx {
        let line = lines[i];
        if let Some(colon) = line.find(':') {
            let key = line[..colon].trim();
            // Skip indented lines (part of nested YAML)
            if key.is_empty() || line.starts_with(' ') || line.starts_with('\t') {
                i += 1;
                continue;
            }
            let mut value = line[colon + 1..].trim().to_string();

            // Handle multi-line list values (key:\n  - item1\n  - item2)
            if value.is_empty() && i + 1 < end_idx {
                let next = lines.get(i + 1).unwrap_or(&"");
                if next.trim_start().starts_with("- ") {
                    let mut items = Vec::new();
                    let mut j = i + 1;
                    while j < end_idx {
                        let item_line = lines[j].trim();
                        if item_line.starts_with("- ") {
                            let item = item_line[2..].trim();
                            let item = item.trim_matches('"').trim_matches('\'');
                            items.push(item.to_string());
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    value = items.join(", ");
                    i = j;
                    if !key.is_empty() {
                        props.insert(key.to_string(), value);
                    }
                    continue;
                }
            }

            // Strip surrounding quotes
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = value[1..value.len() - 1].to_string();
            }

            // Handle inline list [a, b, c]
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                let items: Vec<&str> = inner.split(',').map(|s| {
                    s.trim().trim_matches('"').trim_matches('\'')
                }).collect();
                value = items.join(", ");
            }

            if !key.is_empty() {
                props.insert(key.to_string(), value);
            }
        }
        i += 1;
    }

    Some(props)
}

// ─── Tauri Commands ───

#[tauri::command]
pub fn parse_base_file(app: tauri::AppHandle, file_path: String) -> Result<BaseDefinition, String> {
    // Security: validate path is within a library or the active universe bases dir
    validate_base_path(&app, &file_path)?;

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read base file: {}", e))?;

    // Parse YAML
    serde_json::from_str::<BaseDefinition>(&content)
        .or_else(|_| {
            // Try parsing as YAML (simple key: value format)
            parse_base_yaml(&content)
        })
        .map_err(|e| format!("Failed to parse base file: {}", e))
}

/// Simple YAML-like parser for .base files.
/// For MVP, we use serde_json after converting YAML to JSON.
/// In production, add serde_yaml dependency.
fn parse_base_yaml(content: &str) -> Result<BaseDefinition, String> {
    // For now, try to parse as JSON first (the frontend will save as JSON)
    serde_json::from_str(content)
        .map_err(|e| format!("Invalid base file format: {}", e))
}

/// MIG-065 §I-b — the minimal `LensDefinition` a freshly-created `.base` holds:
/// one clickable name column, table view, the chosen scope (all libraries, or a
/// subset). Serialized to the canonical YAML the unified engine (`execute_lens`
/// / `BaseTab`) reads — the same shape `update_base_columns` round-trips.
/// Replaces the old `BaseDefinition` JSON, which `BaseTab` could not parse.
fn minimal_base_yaml(display_name: String, libraries: Vec<String>) -> Result<String, String> {
    let def = LensDefinition {
        schema: 1,
        lens: display_name,
        template: None,
        scope: LensScope {
            libraries: if libraries.is_empty() {
                LibrariesSelector::All
            } else {
                LibrariesSelector::Subset(libraries)
            },
            federation: FederationMode::Auto,
        },
        where_clauses: vec![],
        order: vec![],
        columns: vec![LensColumn {
            dimension: "note.name".to_string(),
        }],
        view: LensView::Table,
    };
    serde_yaml::to_string(&def).map_err(|e| format!("Failed to serialize base: {}", e))
}

/// Map an old MVP filter operator to the new `prop.*` text-filter op. Numeric
/// `gt`/`lt` have no equivalent in the v1 frontmatter text filters → dropped.
fn convert_filter_op(old: &str) -> Option<&'static str> {
    match old {
        "is" => Some("is"),
        "is_not" => Some("is_not"),
        "contains" => Some("contains"),
        "not_contains" => Some("does_not_contain"),
        "is_empty" => Some("is_empty"),
        "is_not_empty" => Some("is_not_empty"),
        _ => None,
    }
}

/// MIG-065 — convert an OLD Constellation `.base` (the MVP's `BaseDefinition`
/// JSON) to the new `LensDefinition` YAML. With `write = true`, upgrades the
/// file in place — only after the user explicitly chooses to convert (the file
/// is otherwise left untouched). Returns the translated YAML (also used for a
/// read-only preview when `write = false`). A foreign/non-Constellation base
/// fails the `BaseDefinition` parse → Err (caller shows the calm notice). The
/// old columns/filters/sorts (frontmatter keys) become `prop.<key>` dimensions;
/// `note.name` is prepended as the clickable first column.
#[tauri::command]
pub fn convert_base(app: tauri::AppHandle, file_path: String, write: bool) -> Result<String, String> {
    validate_base_path(&app, &file_path)?;
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read base file: {}", e))?;
    let old: BaseDefinition = serde_json::from_str(&content)
        .map_err(|_| "This isn't a convertible Constellation base.".to_string())?;

    let mut columns = vec![LensColumn {
        dimension: "note.name".to_string(),
    }];
    for c in &old.columns {
        columns.push(LensColumn {
            dimension: format!("prop.{}", c.property),
        });
    }
    let order: Vec<LensSort> = old
        .sorts
        .iter()
        .map(|s| LensSort {
            dimension: format!("prop.{}", s.property),
            direction: if s.direction.eq_ignore_ascii_case("desc") {
                SortDirection::Desc
            } else {
                SortDirection::Asc
            },
        })
        .collect();
    let where_clauses: Vec<LensFilter> = old
        .filters
        .iter()
        .filter_map(|f| {
            convert_filter_op(&f.operator).map(|op| LensFilter {
                dimension: format!("prop.{}", f.property),
                op: op.to_string(),
                value: f.value.clone(),
            })
        })
        .collect();
    let libraries = if old.source.selected_vaults.is_empty() {
        LibrariesSelector::All
    } else {
        LibrariesSelector::Subset(old.source.selected_vaults.clone())
    };
    let def = LensDefinition {
        schema: 1,
        lens: old.name,
        template: None,
        scope: LensScope {
            libraries,
            federation: FederationMode::Auto,
        },
        where_clauses,
        order,
        columns,
        view: LensView::Table,
    };
    let yaml = serde_yaml::to_string(&def)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    if write {
        fs::write(&file_path, &yaml)
            .map_err(|e| format!("Failed to write base file: {}", e))?;
    }
    Ok(yaml)
}

#[tauri::command]
pub fn create_base(
    app: tauri::AppHandle,
    folder_path: String,
    file_name: String,
) -> Result<String, String> {
    // Validate folder is in a registered library
    let libraries = crate::libraries::load_libraries_pub(&app);
    let folder = Path::new(&folder_path);
    let canon_folder = fs::canonicalize(folder)
        .map_err(|_| "Folder does not exist.".to_string())?;
    let in_library = libraries.iter().any(|v| {
        fs::canonicalize(&v.path)
            .map(|vp| canon_folder.starts_with(vp))
            .unwrap_or(false)
    });
    if !in_library {
        return Err("Access denied: path is not within any registered library.".to_string());
    }
    if !folder.is_dir() {
        return Err("Folder does not exist.".to_string());
    }

    // Sanitize name
    let safe_name = file_name.trim().replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "");
    if safe_name.is_empty() {
        return Err("Invalid file name.".to_string());
    }

    let name = if safe_name.ends_with(".base") {
        safe_name
    } else {
        format!("{}.base", safe_name)
    };

    let file_path = folder.join(&name);
    if file_path.exists() {
        return Err("A file with this name already exists.".to_string());
    }

    // MIG-065 §I-b — a library-folder base defaults to scope: all; the user
    // refines scope / columns in BaseTab.
    let display_name = name.trim_end_matches(".base").to_string();
    let content = minimal_base_yaml(display_name, vec![])?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to create base file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_base_file(app: tauri::AppHandle, file_path: String, definition: BaseDefinition) -> Result<(), String> {
    // Security: validate path is within a library or the active universe bases dir
    validate_base_path(&app, &file_path)?;

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write base file: {}", e))
}

// Note-open-freeze Batch-2 §B2-3 (2026-07-03): `(async)` + the read→rewrite→write
// cycle moved inside `gate_rmw` — the per-path lock covers the WHOLE cycle, so a
// debounced editor save can land before or after the cell edit but never inside
// its window. Reindex stays OUTSIDE the lock (no DB waits under a path lock).
#[tauri::command(async)]
pub fn update_note_property(
    app: tauri::AppHandle,
    file_path: String,
    key: String,
    value: String,
) -> Result<(), String> {
    // Security: validate the path is in one of the ACTIVE universe's OWN
    // libraries (non-recursive — MIG-065 §J: editing must never write to a
    // read-only cUniverse note), and capture the library name so the search
    // index can be refreshed after the write (MIG-065 §H).
    let libraries = crate::libraries::load_libraries(&app);
    let lib_name = libraries.iter().find(|v| {
        fs::canonicalize(&file_path).ok()
            .and_then(|fp| fs::canonicalize(&v.path).ok().map(|vp| fp.starts_with(vp)))
            .unwrap_or(false)
    }).map(|v| v.name.clone());
    let Some(lib_name) = lib_name else {
        return Err("Access denied: file is not in a registered library.".to_string());
    };

    // MIG-076 §A2 + Batch-2: read-modify-write as ONE gated critical section.
    crate::write_gate::gate_rmw(Path::new(&file_path), "base_edit_cell", |content| {
        Ok(Some(update_frontmatter_property(content, &key, &value)))
    })?;

    // MIG-065 §H — refresh the search index so the Base table (and any later
    // sort / add-column re-query, which reads `note_meta` — not the file)
    // reflects the edit immediately. Best-effort: the disk write is the source
    // of truth; a reindex glitch must not fail the edit (the watcher / next
    // full reindex would catch it anyway).
    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        let _ = crate::search::reindex_single_note(&search_state, &file_path, &lib_name);
    }
    Ok(())
}

/// Update or insert a single property in a note's YAML frontmatter.
fn update_frontmatter_property(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if !content.starts_with("---") {
        // No frontmatter — create one
        let mut result = format!("---\n{}: {}\n---\n", key, format_yaml_value(value));
        result.push_str(content);
        return result;
    }

    let end_idx = lines.iter().skip(1).position(|l| l.trim() == "---");
    let end_idx = match end_idx {
        Some(i) => i + 1,
        None => {
            // Malformed frontmatter — prepend new one
            let mut result = format!("---\n{}: {}\n---\n", key, format_yaml_value(value));
            result.push_str(content);
            return result;
        }
    };

    // Check if property already exists
    let mut found = false;
    let mut new_lines: Vec<String> = Vec::new();
    new_lines.push("---".to_string());

    let mut i = 1;
    while i < end_idx {
        let line = lines[i];
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim();
            if !k.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                if k == key {
                    // Replace existing value
                    new_lines.push(format!("{}: {}", key, format_yaml_value(value)));
                    found = true;
                    // Skip any continuation lines (multi-line list)
                    i += 1;
                    while i < end_idx && (lines[i].starts_with("  - ") || lines[i].starts_with("  ")) {
                        if lines[i].trim().starts_with("- ") {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        new_lines.push(line.to_string());
        i += 1;
    }

    if !found {
        new_lines.push(format!("{}: {}", key, format_yaml_value(value)));
    }

    new_lines.push("---".to_string());

    // Append body (everything after frontmatter)
    for line in &lines[end_idx + 1..] {
        new_lines.push(line.to_string());
    }

    new_lines.join("\n")
}

// ─── Workspace-level Base Storage ───

/// Get the workspace bases directory: {active_universe}/.constellation/bases/
fn workspace_bases_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    let bases_dir = cdir.join("bases");
    fs::create_dir_all(&bases_dir).map_err(|e| format!("Failed to create bases dir: {}", e))?;
    Ok(bases_dir)
}

/// Workspace base entry returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBaseEntry {
    pub id: String,        // file stem (e.g. "My Research")
    pub name: String,      // display name from definition
    pub path: String,      // full file path
    pub modified: u64,     // last modified timestamp
    /// MIG-062 — `None` for the active universe; `Some(name)` for a federated
    /// cUniverse. The sidebar groups entries by this into collapsible
    /// per-universe sub-groups. Read-only federation: a cUniverse's bases are
    /// displayed, never written/moved/deleted (detach is lossless).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub universe_name: Option<String>,
}

/// MIG-062 §D — scan ONE bases directory READ-ONLY (no create_dir_all).
/// Returns entries tagged with `universe_name`. Missing/unreadable dir →
/// empty Vec (non-fatal). Critical: this never writes into the directory,
/// so federating over cUniverse bases dirs cannot mutate a cUniverse.
fn scan_bases_dir(dir: &std::path::Path, universe_name: Option<String>) -> Vec<WorkspaceBaseEntry> {
    let mut entries = Vec::new();
    let Ok(read) = fs::read_dir(dir) else {
        return entries; // missing/unreadable — skip (read-only, non-fatal)
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "base").unwrap_or(false) {
            let id = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Try to read the name from the definition
            let name = fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str::<BaseDefinition>(&c).ok())
                .map(|d| d.name)
                .unwrap_or_else(|| id.clone());

            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            entries.push(WorkspaceBaseEntry {
                id,
                name,
                path: path.to_string_lossy().to_string(),
                modified,
                universe_name: universe_name.clone(),
            });
        }
    }
    entries
}

#[tauri::command]
pub fn list_workspace_bases(app: tauri::AppHandle) -> Result<Vec<WorkspaceBaseEntry>, String> {
    // Active universe — its bases dir IS created if missing (original
    // behavior preserved via workspace_bases_dir). universe_name = None.
    let active_dir = workspace_bases_dir(&app)?;
    let mut entries = scan_bases_dir(&active_dir, None);

    // MIG-062 §D — federate READ-ONLY over the cUniverse tree. Each
    // cUniverse's bases are read from its OWN .constellation/bases/ — with
    // NO create_dir_all, so we never write into a cUniverse. Detaching a
    // cUniverse leaves its bases intact ("the wheel is already there").
    if let Ok(active_root) = crate::universe::active_universe_dir(&app) {
        for cu_root in crate::universe::resolve_child_universe_roots_recursive(&active_root) {
            let cu_name = crate::universe::universe_display_name(&cu_root);
            let cu_bases = crate::universe::constellation_dir(&cu_root).join("bases");
            entries.extend(scan_bases_dir(&cu_bases, Some(cu_name)));
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

#[tauri::command]
pub fn create_workspace_base(
    app: tauri::AppHandle,
    file_name: String,
    selected_libraries: Vec<String>,
) -> Result<String, String> {
    let dir = workspace_bases_dir(&app)?;

    // Sanitize name
    let safe_name = file_name.trim().replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "");
    if safe_name.is_empty() {
        return Err("Invalid file name.".to_string());
    }

    let name = if safe_name.ends_with(".base") {
        safe_name
    } else {
        format!("{}.base", safe_name)
    };

    let file_path = dir.join(&name);
    if file_path.exists() {
        return Err("A base with this name already exists.".to_string());
    }

    // MIG-065 §I-b — write a minimal LensDefinition YAML scoped to the chosen
    // libraries (empty = all), so the sidebar "New Base" opens in BaseTab.
    let display_name = name.trim_end_matches(".base").to_string();
    let content = minimal_base_yaml(display_name, selected_libraries)?;
    fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to create workspace base: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
    definition: BaseDefinition,
) -> Result<(), String> {
    // Validate the path is inside the workspace bases directory
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Invalid workspace bases directory.".to_string())?;
    // For new files that don't exist yet, canonicalize the parent directory and
    // append only the filename — avoids raw-path starts_with bypass via ".." components.
    let canon_target = if target.exists() {
        fs::canonicalize(target)
            .map_err(|_| "Invalid target path.".to_string())?
    } else {
        let parent = target.parent().ok_or("Invalid target path.".to_string())?;
        let canon_parent = fs::canonicalize(parent)
            .map_err(|_| "Parent directory does not exist.".to_string())?;
        let fname = target.file_name().ok_or("Invalid file name.".to_string())?;
        canon_parent.join(fname)
    };

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write workspace base: {}", e))
}

#[tauri::command]
pub fn delete_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<(), String> {
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);

    // Validate path is inside workspace bases directory
    let canon_target = fs::canonicalize(target)
        .map_err(|_| "File does not exist.".to_string())?;
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Workspace directory not found.".to_string())?;

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete workspace base: {}", e))
}

#[tauri::command]
pub fn parse_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<BaseDefinition, String> {
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);

    // Validate path is inside workspace bases directory
    let canon_target = fs::canonicalize(target)
        .map_err(|_| "File does not exist.".to_string())?;
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Workspace directory not found.".to_string())?;

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read workspace base: {}", e))?;

    serde_json::from_str::<BaseDefinition>(&content)
        .map_err(|e| format!("Failed to parse workspace base: {}", e))
}

/// Format a value for YAML output.
/// Quotes strings that contain special characters.
fn format_yaml_value(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    // Check if value needs quoting
    if value.contains(':') || value.contains('#') || value.contains('\'')
        || value.contains('"') || value.contains('\n') || value.starts_with(' ')
        || value.ends_with(' ') || value.starts_with('[') || value.starts_with('{')
    {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}
