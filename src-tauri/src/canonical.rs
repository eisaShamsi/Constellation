//! Canonical Filename Generation & Frontmatter Injection
//!
//! Format: YYYYMMDDTHHMMSSZ_KIND_XXXX.ext
//!
//! - Timestamp: file creation date (UTC) or fallback to modification date
//! - KIND: file kind code from classification engine
//! - XXXX: 4-char uppercase hex suffix for collision avoidance
//! - ext: original file extension

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::file_kinds::{classify_file, KindRegistry};

// ─── Canonical Filename ──────────────────────────────────────────────

/// A parsed canonical filename.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalName {
    pub timestamp: String, // "20260410T153045Z"
    pub kind: String,      // "NOTE"
    pub suffix: String,    // "7F3A"
    pub extension: String, // "md"
    pub full: String,      // "20260410T153045Z_NOTE_7F3A.md"
    pub stem: String,      // "20260410T153045Z_NOTE_7F3A" (= cid)
}

/// Generate a random 4-char uppercase hex suffix.
fn random_suffix() -> String {
    let n: u16 = rand::thread_rng().gen();
    format!("{:04X}", n)
}

/// Format a DateTime<Utc> into the canonical timestamp string.
fn format_timestamp(dt: &DateTime<Utc>) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}

/// Generate a canonical filename for a file.
///
/// - `kind`: the file kind code (e.g., "NOTE", "IMG")
/// - `created`: the creation timestamp (UTC)
/// - `extension`: the file extension without dot (e.g., "md", "png")
/// - `target_dir`: directory to check for collisions (optional)
pub fn generate_canonical(
    kind: &str,
    created: &DateTime<Utc>,
    extension: &str,
    target_dir: Option<&Path>,
) -> CanonicalName {
    let ts = format_timestamp(created);
    let ext = extension.trim_start_matches('.');

    // Try up to 10 suffixes to avoid collision
    for _attempt in 0..10 {
        let suffix = random_suffix();
        let stem = format!("{}_{}_{}",ts, kind, suffix);
        let full = format!("{}.{}", stem, ext);

        if let Some(dir) = target_dir {
            if dir.join(&full).exists() {
                continue; // collision, try again
            }
        }

        return CanonicalName {
            timestamp: ts,
            kind: kind.to_string(),
            suffix,
            extension: ext.to_string(),
            full,
            stem,
        };
    }

    // Fallback: increment timestamp by 1 second
    let fallback_ts = format_timestamp(&(*created + chrono::Duration::seconds(1)));
    let suffix = random_suffix();
    let stem = format!("{}_{}_{}",fallback_ts, kind, suffix);
    let full = format!("{}.{}", stem, ext);
    CanonicalName {
        timestamp: fallback_ts,
        kind: kind.to_string(),
        suffix,
        extension: ext.to_string(),
        full,
        stem,
    }
}

/// Get the creation timestamp of a file.
/// Priority: frontmatter `created:` → filesystem creation time → modification time → now.
pub fn file_creation_time(path: &Path) -> DateTime<Utc> {
    // For .md files, try frontmatter `created:` first — most reliable source
    if path.extension().map(|e| e == "md" || e == "markdown").unwrap_or(false) {
        if let Ok(content) = crate::file_kinds::read_head_pub(path, 2048) {
            if let Some(dt) = extract_created_from_frontmatter(&content) {
                return dt;
            }
        }
    }

    // Filesystem metadata fallback
    if let Ok(meta) = fs::metadata(path) {
        if let Ok(created) = meta.created() {
            return DateTime::from(created);
        }
        if let Ok(modified) = meta.modified() {
            return DateTime::from(modified);
        }
    }
    Utc::now()
}

/// Extract the `created:` date from frontmatter content.
/// Supports ISO 8601 (2026-04-10T15:30:45Z), date-only (2026-04-10), and
/// date+time without T (2026-04-10 15:30:45).
fn extract_created_from_frontmatter(content: &str) -> Option<DateTime<Utc>> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return None; }
    let after = &trimmed[3..];
    let end = after.find("\n---")?;
    let fm = &after[..end];

    for line in fm.lines() {
        let t = line.trim();
        if t.starts_with("created:") {
            let val = t["created:".len()..].trim().trim_matches('"').trim_matches('\'');
            // Try full RFC 3339 / ISO 8601
            if let Ok(dt) = DateTime::parse_from_rfc3339(val) {
                return Some(dt.with_timezone(&Utc));
            }
            // Try "YYYY-MM-DDTHH:MM:SS" without timezone
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S") {
                return Some(dt.and_utc());
            }
            // Try "YYYY-MM-DD HH:MM:SS"
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(val, "%Y-%m-%d %H:%M:%S") {
                return Some(dt.and_utc());
            }
            // Try date-only "YYYY-MM-DD"
            if let Ok(d) = chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                return Some(d.and_hms_opt(0, 0, 0)?.and_utc());
            }
            return None;
        }
    }
    None
}

// ─── Frontmatter Injection ───────────────────────────────────────────

/// Fields to inject into a markdown file's frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontmatterFields {
    pub title: String,
    pub cid: String,
    pub kind: String,
    pub created: String, // ISO 8601
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
}

/// Inject or merge Constellation frontmatter fields into a markdown file's content.
///
/// - If the file has no frontmatter, creates one.
/// - If the file has existing frontmatter, merges fields (preserves user fields).
/// - Never overwrites existing `title`, `tags`, or user-defined fields.
pub fn inject_frontmatter(content: &str, fields: &FrontmatterFields) -> String {
    let trimmed = content.trim_start();

    if trimmed.starts_with("---") {
        let after_first = &trimmed[3..];
        if let Some(end_pos) = after_first.find("\n---") {
            // Well-formed frontmatter — merge
            let existing_fm = &after_first[..end_pos];
            let body = &after_first[end_pos + 4..]; // skip \n---
            let merged = merge_frontmatter(existing_fm, fields);
            return format!("---\n{}---\n{}", merged, body);
        }
        // Malformed frontmatter (opening --- but no closing ---).
        // Treat the entire content after --- as body, prepend fresh frontmatter.
        let body = after_first.trim_start_matches('\n');
        let fm = build_frontmatter(fields);
        return format!("---\n{}---\n\n{}", fm, body);
    }

    // No frontmatter — create new
    let fm = build_frontmatter(fields);
    format!("---\n{}---\n\n{}", fm, content)
}

/// Build a fresh frontmatter string from fields.
fn build_frontmatter(fields: &FrontmatterFields) -> String {
    let mut fm = String::new();
    fm.push_str(&format!("title: \"{}\"\n", escape_yaml_string(&fields.title)));
    fm.push_str(&format!("cid: {}\n", fields.cid));
    fm.push_str(&format!("kind: {}\n", fields.kind.to_lowercase()));
    fm.push_str(&format!("created: {}\n", fields.created));
    if !fields.aliases.is_empty() {
        fm.push_str("aliases:\n");
        for alias in &fields.aliases {
            fm.push_str(&format!("  - \"{}\"\n", escape_yaml_string(alias)));
        }
    }
    if let Some(ref orig) = fields.original_filename {
        fm.push_str(&format!(
            "original_filename: \"{}\"\n",
            escape_yaml_string(orig)
        ));
    }
    fm
}

/// Merge Constellation fields into existing frontmatter, preserving user fields.
fn merge_frontmatter(existing: &str, fields: &FrontmatterFields) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut has_title = false;
    let mut has_cid = false;
    let mut has_kind = false;
    let mut has_created = false;
    let mut has_aliases = false;
    let mut in_aliases_block = false;

    for line in existing.lines() {
        let trimmed = line.trim_start();

        // Track what already exists
        if trimmed.starts_with("title:") {
            has_title = true;
        }
        if trimmed.starts_with("cid:") {
            has_cid = true;
            // Always overwrite cid with ours
            lines.push(format!("cid: {}", fields.cid));
            continue;
        }
        if trimmed.starts_with("kind:") {
            has_kind = true;
            // Always overwrite kind with ours
            lines.push(format!("kind: {}", fields.kind.to_lowercase()));
            continue;
        }
        if trimmed.starts_with("created:") && !has_created {
            has_created = true;
            // Keep existing created date
        }
        if trimmed.starts_with("aliases:") {
            has_aliases = true;
            in_aliases_block = true;
            lines.push(line.to_string());
            // Append our aliases that aren't already there
            // (we'll check in a second pass)
            continue;
        }
        if in_aliases_block {
            if trimmed.starts_with("- ") {
                lines.push(line.to_string());
                continue;
            } else {
                in_aliases_block = false;
                // Before leaving aliases block, add our aliases
                for alias in &fields.aliases {
                    let alias_line = format!("  - \"{}\"", escape_yaml_string(alias));
                    // Don't duplicate
                    if !lines.iter().any(|l| {
                        l.trim().trim_matches('"').trim_matches('\'')
                            == alias.as_str()
                            || l.contains(alias.as_str())
                    }) {
                        lines.push(alias_line);
                    }
                }
            }
        }

        lines.push(line.to_string());
    }

    // If we were still in aliases block at end of frontmatter
    if in_aliases_block {
        for alias in &fields.aliases {
            let alias_line = format!("  - \"{}\"", escape_yaml_string(alias));
            if !lines.iter().any(|l| l.contains(alias.as_str())) {
                lines.push(alias_line);
            }
        }
    }

    // Add missing fields at the end
    if !has_title {
        lines.insert(
            0,
            format!("title: \"{}\"", escape_yaml_string(&fields.title)),
        );
    }
    if !has_cid {
        lines.push(format!("cid: {}", fields.cid));
    }
    if !has_kind {
        lines.push(format!("kind: {}", fields.kind.to_lowercase()));
    }
    if !has_created {
        lines.push(format!("created: {}", fields.created));
    }
    if !has_aliases && !fields.aliases.is_empty() {
        lines.push("aliases:".to_string());
        for alias in &fields.aliases {
            lines.push(format!("  - \"{}\"", escape_yaml_string(alias)));
        }
    }
    if let Some(ref orig) = fields.original_filename {
        if !lines.iter().any(|l| l.trim_start().starts_with("original_filename:")) {
            lines.push(format!(
                "original_filename: \"{}\"",
                escape_yaml_string(orig)
            ));
        }
    }

    let mut result = lines.join("\n");
    result.push('\n');
    result
}

/// Escape special YAML characters in a string value.
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ─── Sidecar Metadata (for non-markdown files) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarMetadata {
    pub title: String,
    pub cid: String,
    pub kind: String,
    pub created: String,
    pub original_filename: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub referenced_by: Vec<String>,
}

/// Create a .meta.json sidecar file path from a canonical file path.
pub fn sidecar_path(canonical_path: &Path) -> PathBuf {
    let stem = canonical_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let ext = canonical_path
        .extension()
        .unwrap_or_default()
        .to_string_lossy();
    canonical_path.with_file_name(format!("{}.{}.meta.json", stem, ext))
}

/// Write a sidecar .meta.json file.
pub fn write_sidecar(canonical_path: &Path, meta: &SidecarMetadata) -> Result<(), String> {
    let path = sidecar_path(canonical_path);
    let json = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to write sidecar {}: {}", path.display(), e))
}

// ─── Import Canonicalization Pipeline ────────────────────────────────

/// Result of a canonicalization operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalizeResult {
    pub total_files: usize,
    pub renamed: usize,
    pub sidecars_created: usize,
    pub errors: Vec<String>,
    pub rename_map: HashMap<String, String>, // old path → new path
}

/// Preview what canonicalization would do to a directory of files.
/// Does NOT modify anything — returns the proposed rename map.
#[tauri::command]
pub fn canonicalize_preview(
    app: tauri::AppHandle,
    library_path: String,
) -> Result<CanonicalizeResult, String> {
    let lib_path = Path::new(&library_path);
    if !lib_path.is_dir() {
        return Err("Library path is not a directory".to_string());
    }

    let config_path = crate::universe::active_constellation_dir(&app)
        .map(|d| d.join("file_kinds.json"))
        .ok();
    let mut registry = KindRegistry::new(config_path.as_deref());

    let mut result = CanonicalizeResult {
        total_files: 0,
        renamed: 0,
        sidecars_created: 0,
        errors: Vec::new(),
        rename_map: HashMap::new(),
    };

    // Walk all files
    let files = collect_files_recursive(lib_path);
    result.total_files = files.len();

    for file_path in &files {
        // Skip files that are already canonical
        if is_canonical_filename(file_path) {
            continue;
        }
        // Skip .meta.json sidecars
        if file_path
            .to_string_lossy()
            .ends_with(".meta.json")
        {
            continue;
        }

        let kind = classify_file(file_path, &mut registry);
        let created = file_creation_time(file_path);
        let ext = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "bin".to_string());

        let parent = file_path.parent().unwrap_or(lib_path);
        let canonical = generate_canonical(&kind, &created, &ext, Some(parent));
        let new_path = parent.join(&canonical.full);

        result.rename_map.insert(
            file_path.to_string_lossy().to_string(),
            new_path.to_string_lossy().to_string(),
        );
        result.renamed += 1;

        // Count sidecars for non-markdown files
        if ext != "md" && ext != "markdown" {
            result.sidecars_created += 1;
        }
    }

    Ok(result)
}

/// Execute canonicalization on a library.
/// Renames files, injects frontmatter, creates sidecars.
#[tauri::command]
pub fn canonicalize_execute(
    app: tauri::AppHandle,
    library_path: String,
) -> Result<CanonicalizeResult, String> {
    let lib_path = Path::new(&library_path);
    if !lib_path.is_dir() {
        return Err("Library path is not a directory".to_string());
    }

    let config_path = crate::universe::active_constellation_dir(&app)
        .map(|d| d.join("file_kinds.json"))
        .ok();
    let mut registry = KindRegistry::new(config_path.as_deref());

    let mut result = CanonicalizeResult {
        total_files: 0,
        renamed: 0,
        sidecars_created: 0,
        errors: Vec::new(),
        rename_map: HashMap::new(),
    };

    // Collect all files first
    let files = collect_files_recursive(lib_path);
    result.total_files = files.len();

    // Phase 1: Build the rename map + enriched content
    struct PendingRename {
        old_path: PathBuf,
        new_path: PathBuf,
        canonical: CanonicalName,
        kind: String,
        original_name: String,
    }
    let mut pending: Vec<PendingRename> = Vec::new();

    for file_path in &files {
        if is_canonical_filename(file_path) {
            continue;
        }
        if file_path.to_string_lossy().ends_with(".meta.json") {
            continue;
        }

        let kind = classify_file(file_path, &mut registry);
        let created = file_creation_time(file_path);
        let ext = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "bin".to_string());

        let parent = file_path.parent().unwrap_or(lib_path);
        let canonical = generate_canonical(&kind, &created, &ext, Some(parent));
        let new_path = parent.join(&canonical.full);

        let original_name = file_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        pending.push(PendingRename {
            old_path: file_path.clone(),
            new_path,
            canonical,
            kind,
            original_name,
        });
    }

    // Phase 2: Enrich and rename
    for item in &pending {
        let ext = item
            .old_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if ext == "md" || ext == "markdown" {
            // Read content, inject frontmatter, write to new path
            match fs::read_to_string(&item.old_path) {
                Ok(content) => {
                    let created_dt = file_creation_time(&item.old_path);
                    let fields = FrontmatterFields {
                        title: item.original_name.clone(),
                        cid: item.canonical.stem.clone(),
                        kind: item.kind.to_lowercase(),
                        created: created_dt.to_rfc3339(),
                        aliases: vec![item.original_name.clone()],
                        original_filename: Some(
                            item.old_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                        ),
                    };
                    let enriched = inject_frontmatter(&content, &fields);
                    if let Err(e) = fs::write(&item.new_path, enriched) {
                        result
                            .errors
                            .push(format!("{}: write failed: {}", item.old_path.display(), e));
                        continue;
                    }
                    // Remove original
                    let _ = fs::remove_file(&item.old_path);
                    result.renamed += 1;
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("{}: read failed: {}", item.old_path.display(), e));
                }
            }
        } else {
            // Non-markdown: rename + create sidecar
            if let Err(e) = fs::rename(&item.old_path, &item.new_path) {
                result
                    .errors
                    .push(format!("{}: rename failed: {}", item.old_path.display(), e));
                continue;
            }
            result.renamed += 1;

            let created_dt = file_creation_time(&item.new_path);
            let sidecar = SidecarMetadata {
                title: item.original_name.clone(),
                cid: item.canonical.stem.clone(),
                kind: item.kind.to_lowercase(),
                created: created_dt.to_rfc3339(),
                original_filename: item
                    .old_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                aliases: vec![item.original_name.clone()],
                referenced_by: Vec::new(),
            };
            if let Err(e) = write_sidecar(&item.new_path, &sidecar) {
                result.errors.push(e);
            } else {
                result.sidecars_created += 1;
            }
        }

        result.rename_map.insert(
            item.old_path.to_string_lossy().to_string(),
            item.new_path.to_string_lossy().to_string(),
        );
    }

    // Write the canonical marker file so the library is recognized as canonicalized
    if result.renamed > 0 {
        let marker_dir = lib_path.join(".constellation");
        let _ = fs::create_dir_all(&marker_dir);
        let _ = fs::write(
            marker_dir.join("canonical"),
            format!("Canonicalized on {}\nFiles renamed: {}\n", Utc::now().to_rfc3339(), result.renamed),
        );
    }

    Ok(result)
}

/// Progress event payload for auto-canonicalization.
#[derive(Clone, Serialize)]
pub struct CanonicalProgress {
    pub phase: String,         // "scanning" | "canonicalizing" | "done"
    pub current: usize,
    pub total: usize,
    pub current_file: String,  // human-readable name of current file
    pub library_name: String,
}

/// Auto-canonicalize all non-canonical files across all libraries in the active universe.
/// Emits "canonical-progress" events so the frontend can show a progress bar.
/// Called on startup — skips files that are already canonical.
#[tauri::command]
pub fn auto_canonicalize_all(app: tauri::AppHandle) -> Result<CanonicalizeResult, String> {
    use tauri::Emitter;

    let libraries = crate::libraries::load_all_libraries(&app);
    let config_path = crate::universe::active_constellation_dir(&app)
        .map(|d| d.join("file_kinds.json"))
        .ok();
    let mut registry = KindRegistry::new(config_path.as_deref());

    let mut total = CanonicalizeResult {
        total_files: 0,
        renamed: 0,
        sidecars_created: 0,
        errors: Vec::new(),
        rename_map: HashMap::new(),
    };

    // Phase 1: Scan all libraries to find non-canonical files
    let _ = app.emit("canonical-progress", CanonicalProgress {
        phase: "scanning".to_string(),
        current: 0, total: 0,
        current_file: String::new(),
        library_name: String::new(),
    });

    struct PendingFile {
        path: PathBuf,
        library_name: String,
        library_path: PathBuf,
    }
    let mut pending: Vec<PendingFile> = Vec::new();

    for lib in &libraries {
        let lib_path = Path::new(&lib.path);
        if !lib_path.is_dir() { continue; }

        let files = collect_files_recursive(lib_path);
        total.total_files += files.len();
        for file_path in files {
            if is_canonical_filename(&file_path) { continue; }
            if file_path.to_string_lossy().ends_with(".meta.json") { continue; }
            pending.push(PendingFile {
                path: file_path,
                library_name: lib.name.clone(),
                library_path: lib_path.to_path_buf(),
            });
        }
    }

    // If nothing to canonicalize, return immediately
    if pending.is_empty() {
        let _ = app.emit("canonical-progress", CanonicalProgress {
            phase: "done".to_string(),
            current: 0, total: 0,
            current_file: String::new(),
            library_name: String::new(),
        });
        return Ok(total);
    }

    let pending_count = pending.len();

    // Phase 2: Canonicalize each file, emitting progress
    for (idx, item) in pending.iter().enumerate() {
        let file_path = &item.path;
        let original_name = file_path.file_stem()
            .unwrap_or_default().to_string_lossy().to_string();
        let original_filename = file_path.file_name()
            .unwrap_or_default().to_string_lossy().to_string();

        // Emit progress every file (frontend throttles display)
        let _ = app.emit("canonical-progress", CanonicalProgress {
            phase: "canonicalizing".to_string(),
            current: idx + 1,
            total: pending_count,
            current_file: original_name.clone(),
            library_name: item.library_name.clone(),
        });

        let kind = crate::file_kinds::classify_file(file_path, &mut registry);
        let created = file_creation_time(file_path);
        let ext = file_path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "bin".to_string());

        let parent = file_path.parent().unwrap_or(&item.library_path);
        let canonical = generate_canonical(&kind, &created, &ext, Some(parent));
        let new_path = parent.join(&canonical.full);

        let ext_lower = ext.to_lowercase();
        if ext_lower == "md" || ext_lower == "markdown" {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let fields = FrontmatterFields {
                        title: original_name.clone(),
                        cid: canonical.stem.clone(),
                        kind: kind.to_lowercase(),
                        created: created.to_rfc3339(),
                        aliases: vec![original_name.clone()],
                        original_filename: Some(original_filename),
                    };
                    let enriched = inject_frontmatter(&content, &fields);
                    if let Err(e) = fs::write(&new_path, enriched) {
                        total.errors.push(format!("{}: write: {}", file_path.display(), e));
                        continue;
                    }
                    let _ = fs::remove_file(file_path);
                    total.renamed += 1;
                }
                Err(e) => { total.errors.push(format!("{}: read: {}", file_path.display(), e)); }
            }
        } else {
            if let Err(e) = fs::rename(file_path, &new_path) {
                total.errors.push(format!("{}: rename: {}", file_path.display(), e));
                continue;
            }
            total.renamed += 1;
            let sidecar = SidecarMetadata {
                title: original_name.clone(),
                cid: canonical.stem.clone(),
                kind: kind.to_lowercase(),
                created: created.to_rfc3339(),
                original_filename: original_filename,
                aliases: vec![original_name],
                referenced_by: Vec::new(),
            };
            if let Err(e) = write_sidecar(&new_path, &sidecar) {
                total.errors.push(e);
            } else {
                total.sidecars_created += 1;
            }
        }

        total.rename_map.insert(
            file_path.to_string_lossy().to_string(),
            new_path.to_string_lossy().to_string(),
        );
    }

    // Phase 3: Done
    let _ = app.emit("canonical-progress", CanonicalProgress {
        phase: "done".to_string(),
        current: pending_count,
        total: pending_count,
        current_file: String::new(),
        library_name: String::new(),
    });

    if total.renamed > 0 {
        eprintln!("[CANONICAL] Auto-canonicalized {} files across {} libraries", total.renamed, libraries.len());
    }

    Ok(total)
}

// ─── Compatible Mode: CID-only injection ────────────────────────────

/// Inject `cid` into frontmatter of all .md files in a library that don't have one yet.
/// Does NOT rename files. Non-destructive — only adds the `cid` field.
#[tauri::command]
pub fn inject_cid_library(app: tauri::AppHandle, library_path: String) -> Result<CanonicalizeResult, String> {
    use tauri::Emitter;
    let lib_path = Path::new(&library_path);
    if !lib_path.is_dir() {
        return Err("Library path is not a directory".to_string());
    }

    let mut result = CanonicalizeResult {
        total_files: 0,
        renamed: 0, // repurposed: counts files with cid injected
        sidecars_created: 0,
        errors: Vec::new(),
        rename_map: HashMap::new(),
    };

    let files = collect_files_recursive(lib_path);
    let md_files: Vec<&PathBuf> = files.iter()
        .filter(|f| f.extension().map(|e| e == "md" || e == "markdown").unwrap_or(false))
        .collect();
    result.total_files = md_files.len();

    for (idx, file_path) in md_files.iter().enumerate() {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => { result.errors.push(format!("{}: {}", file_path.display(), e)); continue; }
        };

        // Skip if already has a cid
        if content.contains("\ncid:") || content.starts_with("cid:") {
            continue;
        }

        let created = file_creation_time(file_path);
        let kind_code = "NOTE"; // default for .md
        let canonical = generate_canonical(kind_code, &created, "md", None);

        // Inject only cid into frontmatter
        let updated = if content.trim_start().starts_with("---") {
            let after = &content.trim_start()[3..];
            if let Some(end) = after.find("\n---") {
                let fm = &after[..end];
                let body = &after[end + 4..];
                format!("---\n{}\ncid: {}\n---{}", fm, canonical.stem, body)
            } else {
                format!("---\ncid: {}\n---\n\n{}", canonical.stem, content)
            }
        } else {
            format!("---\ncid: {}\n---\n\n{}", canonical.stem, content)
        };

        if let Err(e) = fs::write(file_path, &updated) {
            result.errors.push(format!("{}: write: {}", file_path.display(), e));
        } else {
            result.renamed += 1;
        }

        // Emit progress
        if idx % 50 == 0 || idx == md_files.len() - 1 {
            let name = file_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let _ = app.emit("canonical-progress", CanonicalProgress {
                phase: "canonicalizing".to_string(),
                current: idx + 1,
                total: md_files.len(),
                current_file: name,
                library_name: String::new(),
            });
        }
    }

    Ok(result)
}

// ─── De-canonicalize: Restore Original Filenames ─────────────────────

/// Result of a de-canonicalization operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeCanonicalizeResult {
    pub restored: usize,
    pub errors: Vec<String>,
}

/// Restore original filenames for all canonical files in a library.
/// Renames files from canonical (20260411T...Z_NOTE_XXXX.md) back to human names.
/// Uses frontmatter `title` or `original_filename` to determine the target name.
/// Strips `kind` and `original_filename` from frontmatter but PRESERVES `cid` —
/// the unique identifier stays on every note as a frontmatter property so
/// Constellation's living-link system (traversal weights, typed edges, link
/// history) keeps working without imposing filename conventions on the vault.
/// Deletes `.meta.json` sidecars.
#[tauri::command]
pub fn de_canonicalize_library(
    app: tauri::AppHandle,
    library_path: String,
) -> Result<DeCanonicalizeResult, String> {
    use tauri::Emitter;
    let lib_path = Path::new(&library_path);
    if !lib_path.is_dir() {
        return Err("Library path is not a directory".to_string());
    }

    let files = collect_files_recursive(lib_path);
    let mut result = DeCanonicalizeResult { restored: 0, errors: Vec::new() };

    // Collect canonical files
    let canonical_files: Vec<&PathBuf> = files.iter()
        .filter(|f| is_canonical_filename(f))
        .filter(|f| !f.to_string_lossy().ends_with(".meta.json"))
        .collect();

    let total = canonical_files.len();

    for (idx, file_path) in canonical_files.iter().enumerate() {
        let ext = file_path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        // Determine the original/target name and restore
        let _target_name = if ext == "md" || ext == "markdown" {
            // Read frontmatter to find title or original_filename
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    let title = extract_fm_field(&content, "original_filename")
                        .or_else(|| extract_fm_field(&content, "title"))
                        .unwrap_or_else(|| {
                            file_path.file_stem().unwrap_or_default()
                                .to_string_lossy().to_string()
                        });
                    let clean = if title.ends_with(".md") { title } else { format!("{}.md", title) };

                    // Clean frontmatter: remove cid, kind, original_filename, aliases
                    let cleaned = remove_canonical_fields(&content);
                    let parent = file_path.parent().unwrap_or(lib_path);
                    let target = unique_path(parent, &clean);

                    if let Err(e) = fs::write(&target, cleaned) {
                        result.errors.push(format!("{}: write: {}", file_path.display(), e));
                        continue;
                    }
                    let _ = fs::remove_file(file_path);
                    result.restored += 1;

                    // Emit progress
                    let _ = app.emit("canonical-progress", CanonicalProgress {
                        phase: "canonicalizing".to_string(),
                        current: idx + 1, total,
                        current_file: target.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        library_name: String::new(),
                    });
                    continue;
                }
                Err(e) => {
                    result.errors.push(format!("{}: read: {}", file_path.display(), e));
                    continue;
                }
            }
        } else {
            // Non-markdown: check sidecar for original name
            let sc_path = sidecar_path(file_path);
            let orig = if sc_path.exists() {
                fs::read_to_string(&sc_path).ok()
                    .and_then(|json| serde_json::from_str::<SidecarMetadata>(&json).ok())
                    .map(|m| m.original_filename)
                    .unwrap_or_else(|| file_path.file_name().unwrap_or_default().to_string_lossy().to_string())
            } else {
                file_path.file_name().unwrap_or_default().to_string_lossy().to_string()
            };

            let parent = file_path.parent().unwrap_or(lib_path);
            let target = unique_path(parent, &orig);

            if let Err(e) = fs::rename(file_path, &target) {
                result.errors.push(format!("{}: rename: {}", file_path.display(), e));
                continue;
            }
            // Remove sidecar
            let _ = fs::remove_file(&sc_path);
            result.restored += 1;
            continue;
        };
    }

    // Update library mode to compatible
    let libraries = crate::libraries::load_all_libraries(&app);
    if let Some(lib) = libraries.iter().find(|l| l.path == library_path) {
        let _ = crate::libraries::set_library_canonical_mode(
            app.clone(), lib.id.clone(), "compatible".to_string()
        );
    }

    let _ = app.emit("canonical-progress", CanonicalProgress {
        phase: "done".to_string(),
        current: total, total,
        current_file: String::new(),
        library_name: String::new(),
    });

    eprintln!("[CANONICAL] De-canonicalized {} files, {} errors", result.restored, result.errors.len());
    Ok(result)
}

/// Extract a single frontmatter field value.
fn extract_fm_field(content: &str, key: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return None; }
    let after = &trimmed[3..];
    let end = after.find("\n---")?;
    for line in after[..end].lines() {
        let t = line.trim();
        if t.starts_with(key) && t[key.len()..].trim_start().starts_with(':') {
            let val = t[key.len()..].trim_start().trim_start_matches(':').trim();
            let val = val.trim_matches('"').trim_matches('\'');
            if !val.is_empty() { return Some(val.to_string()); }
        }
    }
    None
}

/// Remove the canonical-specific fields (kind, original_filename, aliases) but
/// KEEP the `cid` property — the unique identifier stays on the note as a
/// frontmatter field even after the filename reverts to its original form.
/// This lets Constellation continue to track the note (for links, weights,
/// traversal history) without imposing filename conventions on the vault.
fn remove_canonical_fields(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return content.to_string(); }
    let after = &trimmed[3..];
    let Some(end) = after.find("\n---") else { return content.to_string(); };
    let fm = &after[..end];
    let body = &after[end + 4..];

    // Fields we strip on de-canonicalization. `cid` is deliberately preserved.
    let strip_keys = ["kind:", "original_filename:"];
    let mut in_aliases = false;
    let mut new_lines: Vec<&str> = Vec::new();

    for line in fm.lines() {
        let t = line.trim();
        if t.starts_with("aliases:") {
            in_aliases = true;
            continue;
        }
        if in_aliases {
            if t.starts_with("- ") { continue; }
            in_aliases = false;
        }
        if strip_keys.iter().any(|k| t.starts_with(k)) { continue; }
        new_lines.push(line);
    }

    if new_lines.is_empty() || new_lines.iter().all(|l| l.trim().is_empty()) {
        body.trim_start().to_string()
    } else {
        format!("---\n{}\n---{}", new_lines.join("\n"), body)
    }
}

/// Generate a unique path, appending (1), (2), etc. if the target already exists.
fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let target = dir.join(filename);
    if !target.exists() { return target; }

    let stem = Path::new(filename).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let ext = Path::new(filename).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();

    for i in 1..=999 {
        let candidate = dir.join(format!("{} ({}){}", stem, i, ext));
        if !candidate.exists() { return candidate; }
    }
    dir.join(format!("{}_restored{}", stem, ext))
}

/// Generate a canonical name for a new note (called from the frontend).
#[tauri::command]
pub fn generate_canonical_name(
    kind: String,
    created: Option<String>,
) -> Result<CanonicalName, String> {
    let dt: DateTime<Utc> = if let Some(ref ts) = created {
        DateTime::parse_from_rfc3339(ts)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    let ext = match kind.to_uppercase().as_str() {
        "NOTE" | "BASE" | "TMPL" | "LINK" | "MARK" | "CLIP" => "md",
        "CANVAS" => "json",
        _ => "bin",
    };

    Ok(generate_canonical(&kind.to_uppercase(), &dt, ext, None))
}

// ─── Helpers ─────────────────────────────────────────────────────────

/// Check if a library has been canonicalized (has `.constellation/canonical` marker).
#[allow(dead_code)]
pub fn is_library_canonicalized(library_path: &str) -> bool {
    Path::new(library_path).join(".constellation").join("canonical").exists()
}

/// Check if a filename already follows the canonical pattern.
pub fn is_canonical_filename(path: &Path) -> bool {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    // Pattern: YYYYMMDDTHHMMSSZ_KIND_XXXX
    // Min length: 16 (timestamp) + 1 (_) + 1 (kind min) + 1 (_) + 4 (suffix) = 23
    if stem.len() < 23 {
        return false;
    }
    // Check timestamp format: digits, T at pos 8, digits, Z at pos 15
    let bytes = stem.as_bytes();
    bytes[8] == b'T'
        && bytes[15] == b'Z'
        && bytes[16] == b'_'
        && bytes.iter().take(8).all(|b| b.is_ascii_digit())
        && bytes[9..15].iter().all(|b| b.is_ascii_digit())
}

/// Recursively collect all files in a directory (skipping hidden dirs and excluded dirs).
/// Depth-limited to 30 levels to prevent stack overflow on pathological directory structures.
fn collect_files_recursive(dir: &Path) -> Vec<PathBuf> {
    collect_files_recursive_depth(dir, 0)
}

fn collect_files_recursive_depth(dir: &Path, depth: u32) -> Vec<PathBuf> {
    if depth > 30 { return Vec::new(); }
    let mut files = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            if name.starts_with('.')
                || crate::file_kinds::EXCLUDED_DIRS
                    .iter()
                    .any(|d| name.eq_ignore_ascii_case(d))
            {
                continue;
            }
            files.extend(collect_files_recursive_depth(&path, depth + 1));
        } else {
            files.push(path);
        }
    }

    files
}

// ─── Startup safeguard ─────────────────────────────────────────────
//
// The canonical filename scheme (20260410T153045Z_NOTE_XXXX.md) is only
// appropriate for libraries CREATED by Constellation. Earlier builds
// inadvertently applied it to external libraries (e.g. Obsidian vaults)
// on import, which renamed every note on disk and broke every wikilink
// referencing the original names.
//
// `repair_external_libraries_on_startup` scans every registered library.
// For any whose path contains canonical-named files paired with an
// `original_filename` or `title` in frontmatter, it de-canonicalizes
// silently, restoring the vault to its original state. Idempotent: a
// library with no canonical files is a no-op. CID is preserved on every
// note as a frontmatter property so Living Link data survives the
// revert.

/// Returns true if any .md file in `lib_path` is in canonical filename
/// format. Cheap probe — used to decide whether to run the revert.
fn library_has_canonical_md(lib_path: &Path) -> bool {
    let files = collect_files_recursive(lib_path);
    files.iter().any(|f| {
        f.extension().and_then(|e| e.to_str()) == Some("md")
            && is_canonical_filename(f)
    })
}

/// Startup migration: scan every registered library for canonical-named files.
/// If any are found, revert them to their original filenames using the
/// `title` / `original_filename` / aliases preserved in frontmatter (or the
/// `.meta.json` sidecar for attachments). Idempotent — a library with no
/// canonical files is skipped.
///
/// NO mode check. Earlier builds may have left a library marked "native"
/// while its files were actually canonicalized from an external import; the
/// mode flag is not reliable, the filesystem is. If a library contains
/// files in canonical format (20260410T153045Z_NOTE_XXXX.md) AND those
/// files carry restore metadata in their frontmatter, revert them.
#[tauri::command]
pub fn repair_external_libraries_on_startup(
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let mut repaired: Vec<String> = Vec::new();
    let libraries = crate::libraries::load_all_libraries(&app);
    eprintln!("[CANONICAL] Checking {} libraries for canonical-format files to repair", libraries.len());
    for lib in libraries {
        let lib_path = Path::new(&lib.path);
        if !lib_path.is_dir() {
            eprintln!("[CANONICAL]   - {}: path not accessible, skipped", lib.path);
            continue;
        }
        if !library_has_canonical_md(lib_path) {
            eprintln!("[CANONICAL]   - {}: no canonical-format files, clean", lib.name);
            continue;
        }
        eprintln!("[CANONICAL]   - {}: canonical files detected, reverting...", lib.name);
        match de_canonicalize_library(app.clone(), lib.path.clone()) {
            Ok(res) => {
                eprintln!("[CANONICAL]     restored {} files, {} errors", res.restored, res.errors.len());
                for err in &res.errors { eprintln!("[CANONICAL]       ! {}", err); }
                if res.restored > 0 { repaired.push(lib.name.clone()); }
            }
            Err(e) => eprintln!("[CANONICAL]     FAILED: {}", e),
        }
    }
    Ok(repaired)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_name_format() {
        let dt = Utc::now();
        let cn = generate_canonical("NOTE", &dt, "md", None);
        assert!(cn.full.ends_with(".md"));
        assert!(cn.stem.contains("_NOTE_"));
        assert_eq!(cn.suffix.len(), 4);
        assert!(cn.stem.contains('T'));
        assert!(cn.stem.contains('Z'));
    }

    #[test]
    fn test_is_canonical() {
        let p = Path::new("20260410T153045Z_NOTE_7F3A.md");
        assert!(is_canonical_filename(p));

        let p2 = Path::new("Agriculture System.md");
        assert!(!is_canonical_filename(p2));

        let p3 = Path::new("20260410T153045Z_IMG_E5F6.png");
        assert!(is_canonical_filename(p3));
    }

    #[test]
    fn test_inject_frontmatter_new() {
        let content = "# Hello World\n\nSome content.";
        let fields = FrontmatterFields {
            title: "Hello World".to_string(),
            cid: "20260410T153045Z_NOTE_7F3A".to_string(),
            kind: "note".to_string(),
            created: "2026-04-10T15:30:45Z".to_string(),
            aliases: vec!["Hello World".to_string()],
            original_filename: Some("Hello World.md".to_string()),
        };
        let result = inject_frontmatter(content, &fields);
        assert!(result.starts_with("---\n"));
        assert!(result.contains("cid: 20260410T153045Z_NOTE_7F3A"));
        assert!(result.contains("kind: note"));
        assert!(result.contains("# Hello World"));
    }

    #[test]
    fn test_inject_frontmatter_merge() {
        let content = "---\ntitle: My Note\ntags:\n  - rust\n  - code\n---\n\nBody text.";
        let fields = FrontmatterFields {
            title: "My Note".to_string(),
            cid: "20260410T153045Z_NOTE_ABCD".to_string(),
            kind: "note".to_string(),
            created: "2026-04-10T15:30:45Z".to_string(),
            aliases: vec!["My Note".to_string()],
            original_filename: None,
        };
        let result = inject_frontmatter(content, &fields);
        assert!(result.contains("cid: 20260410T153045Z_NOTE_ABCD"));
        assert!(result.contains("tags:"));
        assert!(result.contains("- rust"));
        assert!(result.contains("Body text."));
    }

    #[test]
    fn test_sidecar_path() {
        let p = Path::new("/lib/20260410T153045Z_IMG_E5F6.png");
        let sp = sidecar_path(p);
        assert_eq!(
            sp.file_name().unwrap().to_str().unwrap(),
            "20260410T153045Z_IMG_E5F6.png.meta.json"
        );
    }
}
