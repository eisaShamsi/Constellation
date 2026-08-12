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
        // Root key only (2026-08-01 sweep of the 2026-07-21 trimmed-line class): a
        // nested `created:` child belongs to another map, and a block scalar's prose
        // may open with the word — neither is the note's created date. Block-scalar
        // content must be more-indented than its key, so the indentation test covers
        // it (same reliance as update_frontmatter_title).
        if crate::yaml_lines::is_top_level_key_line(line) && t.starts_with("created:") {
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
///
/// PJ-153 (MIG-105 C6): the identity is emitted under the namespaced key
/// `cid_cn:` — NEVER the legacy `cid:`. `index_note`'s extractor reads only
/// `cid_cn:` (search.rs `extract_frontmatter_cid_cn`), so a legacy emission
/// indexed as cid_cn='' — an identity-less note invisible to every
/// cid-keyed surface until first tab-open.
fn build_frontmatter(fields: &FrontmatterFields) -> String {
    let mut fm = String::new();
    fm.push_str(&format!("title: \"{}\"\n", escape_yaml_string(&fields.title)));
    fm.push_str(&format!("cid_cn: {}\n", fields.cid));
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
    // PJ-182 — the indentation the EXISTING alias items use, so an appended one joins
    // their sequence rather than nesting under the last of them.
    let mut alias_indent: Option<String> = None;
    // The ROOT alias block's item VALUES, parsed the way the block is written (dash
    // stripped, surrounding quotes stripped), so the dedup below compares exact
    // values like-for-like. The substring test this replaced (`l.contains(alias)`)
    // silently suppressed a genuinely-absent alias whenever its text merely appeared
    // inside the title line, a longer alias, or a URL (2026-08-01 inspection, LOW).
    let mut existing_alias_values: Vec<String> = Vec::new();
    // PJ-153 (MIG-105 C6): when the block already carries the namespaced
    // `cid_cn:`, any legacy `cid:` line is DROPPED (a block must never end
    // up with two identity keys). Root keys only — a NESTED `cid_cn:` is a
    // user map's child, not the note's identity.
    let existing_has_cid_cn = existing
        .lines()
        .any(|l| crate::yaml_lines::is_top_level_key_line(l) && l.starts_with("cid_cn:"));

    for line in existing.lines() {
        let trimmed = line.trim_start();
        // INDENTATION IS DATA (the 2026-07-21 app-killer class, fixed in
        // update_frontmatter_title and never swept into this sibling): a nested map's
        // `kind:` / `aliases:` / `cid_cn:` child and a block scalar's prose lines are
        // INDENTED, so matching the TRIMMED line treated them as root keys — replacing
        // a nested `kind:` with a column-0 line, opening the alias block inside a
        // foreign map, and letting a nested `cid_cn:` suppress minting the note's real
        // identity. Every key matcher below fires at the root only. (Block-scalar
        // content must be more-indented than its key, so the indentation test excludes
        // it — the same reliance update_frontmatter_title has.)
        let root_key = crate::yaml_lines::is_top_level_key_line(line);

        if in_aliases_block {
            if crate::yaml_lines::is_seq_item(line) {
                // PJ-182 — remember the block's OWN indentation. Appending at a hardcoded
                // "  " into a block whose items sit at column 0 mixes two indentations
                // inside one sequence: the emitted YAML still parses, but the appended
                // line is read as a CONTINUATION of the previous item, so the user's last
                // alias silently becomes `Older Name - "Injected"`.
                if alias_indent.is_none() {
                    alias_indent = Some(crate::yaml_lines::indent_of(line).to_string());
                }
                if let Some(v) = crate::yaml_lines::seq_item_value(line) {
                    existing_alias_values
                        .push(v.trim().trim_matches('"').trim_matches('\'').to_string());
                }
                lines.push(line.to_string());
                continue;
            }
            // Any non-item line ENDS the block, and this must run BEFORE the key
            // branches below (update_frontmatter_title's rule): a root key directly
            // after the items would otherwise be consumed by its own branch with the
            // block still open, and the appended aliases would land AFTER that key —
            // orphaned sequence items under a scalar, YAML that no longer parses.
            in_aliases_block = false;
            let ind = alias_indent.as_deref().unwrap_or("  ");
            for alias in &fields.aliases {
                // Exact value match, never substring.
                if existing_alias_values.iter().any(|v| v == alias) {
                    continue;
                }
                lines.push(format!("{}- \"{}\"", ind, escape_yaml_string(alias)));
                existing_alias_values.push(alias.clone());
            }
            // The line that closed the block falls through to normal processing.
        }

        // Track what already exists
        if root_key && trimmed.starts_with("title:") {
            has_title = true;
        }
        if root_key && trimmed.starts_with("cid_cn:") {
            // Existing namespaced identity — PRESERVED, never re-minted:
            // cid_cn is the note's durable identity; overwriting it would
            // sever every earned row (links, review, history) keyed to it.
            if has_cid {
                continue; // malformed duplicate — keep only the first
            }
            has_cid = true;
            lines.push(line.to_string());
            continue;
        }
        if root_key && trimmed.starts_with("cid:") {
            // Legacy key — rewritten as `cid_cn:` keeping ITS value (the
            // migrate_cid_to_cid_cn transform: identity survives the merge),
            // unless a `cid_cn:` line already owns identity.
            if has_cid || existing_has_cid_cn {
                continue;
            }
            has_cid = true;
            // A root key is unindented, so the rewrite emits at column 0.
            lines.push(format!("cid_cn:{}", &trimmed[4..]));
            continue;
        }
        if root_key && trimmed.starts_with("kind:") {
            has_kind = true;
            // Always overwrite kind with ours
            lines.push(format!("kind: {}", fields.kind.to_lowercase()));
            continue;
        }
        if root_key && trimmed.starts_with("created:") && !has_created {
            has_created = true;
            // Keep existing created date
        }
        if root_key && trimmed.starts_with("aliases:") {
            has_aliases = true;
            in_aliases_block = true;
            lines.push(line.to_string());
            // Append our aliases that aren't already there
            // (checked as the block's items stream past)
            continue;
        }

        lines.push(line.to_string());
    }

    // If we were still in aliases block at end of frontmatter
    if in_aliases_block {
        let ind = alias_indent.as_deref().unwrap_or("  ");
        for alias in &fields.aliases {
            // Exact value match, never substring (same rule as the mid-block site).
            if existing_alias_values.iter().any(|v| v == alias) {
                continue;
            }
            lines.push(format!("{}- \"{}\"", ind, escape_yaml_string(alias)));
            existing_alias_values.push(alias.clone());
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
        // No identity key of either spelling — mint under the namespaced key.
        lines.push(format!("cid_cn: {}", fields.cid));
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
        // Root key only — a nested `original_filename:` child is the user's data and
        // must not suppress recording the note's own.
        if !lines
            .iter()
            .any(|l| crate::yaml_lines::is_top_level_key_line(l) && l.starts_with("original_filename:"))
        {
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
    let nested = crate::libraries::nested_library_paths(&crate::libraries::load_all_libraries(&app), &library_path);
    let files = collect_files_recursive(lib_path, &nested);
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
    let nested = crate::libraries::nested_library_paths(&crate::libraries::load_all_libraries(&app), &library_path);
    let files = collect_files_recursive(lib_path, &nested);
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
                    // MIG-076 §A2 — gated.
                    if let Err(e) = crate::write_gate::gate_write(&item.new_path, &enriched, None, "canonicalize") {
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
            if let Err(e) = crate::write_gate::gate_rename(&item.old_path, &item.new_path, "canonicalize") {
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

    // PJ-235 — OWN libraries only: this file's three commands RENAME the user's files or
    // revert canonicalization, and the federated resolver would reach into a LINKED
    // universe, which Constellation reads but never writes.
    //
    // Call-graph, VERIFIED 2026-08-11 (an earlier revision of this comment claimed
    // `auto_canonicalize_all` runs at startup — it does not, and the panel review caught
    // the false claim before it entered the record): the startup path is
    // `repair_external_libraries_on_startup` (+layout.svelte:3466, localStorage-gated —
    // and localStorage is proven non-durable, PJ-110/PJ-103, so the gate can reset).
    // `auto_canonicalize_all` is registered (lib.rs) but its only wrapper,
    // `autoCanonicalize()` in importers/store.ts, currently has NO caller in src/.
    let libraries = crate::libraries::load_libraries(&app);
    let config_path = crate::universe::active_constellation_dir(&app)
        .map(|d| d.join("file_kinds.json"))
        .ok();
    let foreign_roots = crate::libraries::foreign_library_roots(&app, &libraries);
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

        // PJ-235 (third pass) — **the exclusion set must still know the federation.** Narrowing
        // `libraries` to own-only (right, because this RENAMES files) also narrowed the boundary
        // this walk stops at — so a cUniverse nested UNDER the active root stopped being a
        // boundary and `collect_files_recursive` descended into it and renamed a LINKED
        // universe's files. That is the same mistake made one file over in
        // `list_universe_folders`, repeated here in the same pass and caught by the adversarial
        // review. `walk_exclusions` carries both boundaries; a walker needs both or it is
        // half-scoped.
        let exclude = crate::libraries::walk_exclusions(&libraries, &lib.path, &foreign_roots);
        let files = collect_files_recursive(lib_path, &exclude);
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
                    // MIG-076 §A2 — gated.
                    if let Err(e) = crate::write_gate::gate_write(&new_path, &enriched, None, "import_adopt") {
                        total.errors.push(format!("{}: write: {}", file_path.display(), e));
                        continue;
                    }
                    let _ = fs::remove_file(file_path);
                    total.renamed += 1;
                }
                Err(e) => { total.errors.push(format!("{}: read: {}", file_path.display(), e)); }
            }
        } else {
            if let Err(e) = crate::write_gate::gate_rename(file_path, &new_path, "import_adopt") {
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

    let nested = crate::libraries::nested_library_paths(&crate::libraries::load_all_libraries(&app), &library_path);
    let files = collect_files_recursive(lib_path, &nested);
    let md_files: Vec<&PathBuf> = files.iter()
        .filter(|f| f.extension().map(|e| e == "md" || e == "markdown").unwrap_or(false))
        .collect();
    result.total_files = md_files.len();

    for (idx, file_path) in md_files.iter().enumerate() {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => { result.errors.push(format!("{}: {}", file_path.display(), e)); continue; }
        };

        // Skip if already has a cid_cn (or legacy cid — migration path below)
        if content.contains("\ncid_cn:") || content.starts_with("cid_cn:") {
            continue;
        }
        // Legacy `cid:` migration: if present, rename the key to `cid_cn:` and
        // leave the value intact. Namespacing avoids collision with any
        // pre-existing `cid:` property in the user's vault.
        if content.contains("\ncid:") || content.starts_with("cid:") {
            let migrated = migrate_cid_to_cid_cn(&content);
            if migrated != content {
                if let Err(e) = crate::write_gate::gate_write(file_path, &migrated, None, "cid_migrate_bulk") {
                    result.errors.push(format!("{}: migrate cid→cid_cn: {}", file_path.display(), e));
                } else {
                    result.renamed += 1;
                }
            }
            continue;
        }

        let created = file_creation_time(file_path);
        let kind_code = "NOTE"; // default for .md
        let canonical = generate_canonical(kind_code, &created, "md", None);

        // Inject cid_cn into frontmatter. The user sees this as a regular
        // property in the Properties panel; it's Constellation's stable
        // identifier, namespaced so it cannot collide with any existing
        // `cid:` field in the user's vault.
        let updated = if content.trim_start().starts_with("---") {
            let after = &content.trim_start()[3..];
            if let Some(end) = after.find("\n---") {
                let fm = &after[..end];
                let body = &after[end + 4..];
                format!("---\n{}\ncid_cn: {}\n---{}", fm, canonical.stem, body)
            } else {
                format!("---\ncid_cn: {}\n---\n\n{}", canonical.stem, content)
            }
        } else {
            format!("---\ncid_cn: {}\n---\n\n{}", canonical.stem, content)
        };

        if let Err(e) = crate::write_gate::gate_write(file_path, &updated, None, "cid_inject_bulk") {
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
/// Strips `kind` and `original_filename` from frontmatter but PRESERVES the
/// note's identity — `cid_cn:` stays as-is, and a legacy `cid:` is migrated to
/// `cid_cn:` (value kept) on the same pass (see `remove_canonical_fields`) —
/// so Constellation's living-link system (traversal weights, typed edges, link
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

    let nested = crate::libraries::nested_library_paths(&crate::libraries::load_all_libraries(&app), &library_path);
    let files = collect_files_recursive(lib_path, &nested);
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

                    // Clean frontmatter: strip kind/original_filename/aliases;
                    // KEEP identity (cid_cn preserved, legacy cid: migrated).
                    let cleaned = remove_canonical_fields(&content);
                    let parent = file_path.parent().unwrap_or(lib_path);
                    let target = unique_path(parent, &clean);

                    if let Err(e) = crate::write_gate::gate_write(&target, &cleaned, None, "decanonicalize") {
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
            // Non-markdown: the `.meta.json` sidecar is the ONLY record of the original name.
            //
            // PJ-207 §15 — `fs::read_to_string(&sc_path).ok()` collapsed "the sidecar is
            // unreadable RIGHT NOW" — the everyday Windows sharing violation from OneDrive or
            // Defender, the exact distinction `universe::read_persisted_json` exists to make —
            // into "there is no original name". The CANONICAL filename was then substituted as
            // the "original"; `unique_path` found that name occupied by the file itself and
            // appended " (1)", so the attachment was renamed to a mangled THIRD name that broke
            // every `![[...]]` embed pointing at it; and then the sidecar — the last copy of
            // the user's real filename — was deleted unconditionally while the file counted as
            // `restored` with nothing pushed to `errors`. The same substitution ran for files
            // with no sidecar at all. This path also runs unattended at boot
            // (`repair_external_libraries_on_startup`), so nothing could ever reach the user.
            //
            // Refuse and report beats rewrite: an unreadable, unparseable or absent sidecar now
            // leaves BOTH the file and the sidecar exactly as they are.
            let sc_path = sidecar_path(file_path);
            let current_name = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let orig: Option<String> = if sc_path.exists() {
                match fs::read_to_string(&sc_path) {
                    Ok(json) => match serde_json::from_str::<SidecarMetadata>(&json) {
                        Ok(m) => Some(m.original_filename),
                        Err(e) => {
                            result.errors.push(format!(
                                "{}: sidecar unparseable ({}) — file left as-is; the sidecar is kept for recovery",
                                sc_path.display(), e
                            ));
                            None
                        }
                    },
                    Err(e) => {
                        result.errors.push(format!(
                            "{}: sidecar unreadable ({}) — file left as-is; nothing renamed, nothing deleted",
                            sc_path.display(), e
                        ));
                        None
                    }
                }
            } else {
                result.errors.push(format!(
                    "{}: no .meta.json sidecar — the original filename is unknown, file left as-is",
                    file_path.display()
                ));
                None
            };
            let orig = match orig {
                Some(o) => o,
                None => continue,
            };

            // The sidecar names the file it is already called: nothing to restore, and
            // `unique_path` would rename it onto itself and mangle it to " (1)".
            if orig == current_name {
                if let Err(e) = fs::remove_file(&sc_path) {
                    result.errors.push(format!("{}: sidecar not removed: {}", sc_path.display(), e));
                }
                result.restored += 1;
                continue;
            }

            let parent = file_path.parent().unwrap_or(lib_path);
            let target = unique_path(parent, &orig);

            if let Err(e) = crate::write_gate::gate_rename(file_path, &target, "decanonicalize") {
                result.errors.push(format!("{}: rename: {}", file_path.display(), e));
                continue;
            }
            // Remove sidecar — and say so if it survives, rather than orphaning it silently.
            if let Err(e) = fs::remove_file(&sc_path) {
                result.errors.push(format!("{}: sidecar not removed: {}", sc_path.display(), e));
            }
            result.restored += 1;
            continue;
        };
    }

    // Update library mode to compatible
    // PJ-235 — OWN libraries only: this file's three commands RENAME the user's files or
    // revert canonicalization, and the federated resolver would reach into a LINKED
    // universe, which Constellation reads but never writes.
    //
    // Call-graph, VERIFIED 2026-08-11 (an earlier revision of this comment claimed
    // `auto_canonicalize_all` runs at startup — it does not, and the panel review caught
    // the false claim before it entered the record): the startup path is
    // `repair_external_libraries_on_startup` (+layout.svelte:3466, localStorage-gated —
    // and localStorage is proven non-durable, PJ-110/PJ-103, so the gate can reset).
    // `auto_canonicalize_all` is registered (lib.rs) but its only wrapper,
    // `autoCanonicalize()` in importers/store.ts, currently has NO caller in src/.
    let libraries = crate::libraries::load_libraries(&app);
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
        // Root key only (2026-08-01 sweep of the 2026-07-21 trimmed-line class): an
        // indented match is a nested map's child or block-scalar prose, not the field.
        if crate::yaml_lines::is_top_level_key_line(line)
            && t.starts_with(key)
            && t[key.len()..].trim_start().starts_with(':')
        {
            let val = t[key.len()..].trim_start().trim_start_matches(':').trim();
            let val = val.trim_matches('"').trim_matches('\'');
            if !val.is_empty() { return Some(val.to_string()); }
        }
    }
    None
}

/// Remove the canonical-specific fields (kind, original_filename, aliases) and
/// migrate any legacy `cid:` to `cid_cn:`. Preserves `cid_cn:` — the
/// namespaced stable identifier stays on every note so the Living Link
/// system keeps working after a filename revert without imposing filename
/// conventions on the vault.
fn remove_canonical_fields(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return content.to_string(); }
    let after = &trimmed[3..];
    let Some(end) = after.find("\n---") else { return content.to_string(); };
    let fm = &after[..end];
    let body = &after[end + 4..];

    let strip_keys = ["kind:", "original_filename:"];
    let mut in_aliases = false;
    let mut new_lines: Vec<String> = Vec::new();

    for line in fm.lines() {
        let t = line.trim();
        // Root keys only (2026-08-01 sweep of the 2026-07-21 trimmed-line class —
        // PJ-182 fixed only the seq-item half of this loop): an INDENTED `aliases:` /
        // `kind:` / `original_filename:` / `cid:` is a user map's child (or a block
        // scalar's prose), and stripping it — plus, for `aliases:`, every item under
        // it — destroys the user's data.
        let root_key = crate::yaml_lines::is_top_level_key_line(line);
        if root_key && t.starts_with("aliases:") {
            in_aliases = true;
            continue;
        }
        if in_aliases {
            // PJ-182 — the shared rule, so this sibling of `merge_frontmatter` cannot
            // drift from it on a bare `-` or a tab-separated item.
            if crate::yaml_lines::is_seq_item(line) { continue; }
            in_aliases = false;
        }
        if root_key && strip_keys.iter().any(|k| t.starts_with(k)) { continue; }
        // Migrate legacy `cid:` → `cid_cn:` at the same time. A root key is
        // unindented, so the rewrite emits at column 0.
        if root_key && t.starts_with("cid:") && !t.starts_with("cid_cn") {
            new_lines.push(format!("cid_cn:{}", &t[4..]));
            continue;
        }
        new_lines.push(line.to_string());
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
///
/// MIG-108 Slice 0 — `exclude` is the `nested_library_paths` set: a subdirectory that is
/// itself a REGISTERED LIBRARY is another library's territory, not this one's files
/// ("Library ≠ Folder"). Without the boundary, walking the universe_notes library — whose
/// path IS the universe root — folded every nested library into it: the boot repair probe
/// walked all of them per outer library (quadratic), and post-MIG-108 every library is
/// nested under the root, so this walker would have double-processed the whole universe.
fn collect_files_recursive(dir: &Path, exclude: &std::collections::HashSet<String>) -> Vec<PathBuf> {
    collect_files_recursive_depth(dir, 0, exclude)
}

fn collect_files_recursive_depth(
    dir: &Path,
    depth: u32,
    exclude: &std::collections::HashSet<String>,
) -> Vec<PathBuf> {
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
            if crate::libraries::is_nested_library(&path, exclude) { continue; } // Library ≠ Folder
            files.extend(collect_files_recursive_depth(&path, depth + 1, exclude));
        } else {
            files.push(path);
        }
    }

    files
}

// ─── cid_cn namespace (decision 4) ─────────────────────────────────
//
// Constellation's stable note identifier is stored under the namespaced
// property name `cid_cn:` (Constellation Node id) instead of the generic
// `cid:` so it can never collide with a pre-existing `cid:` property in
// a user's Obsidian vault. Internal helpers migrate any legacy `cid:`
// to `cid_cn:` in-place on first touch.

/// Rename the frontmatter key `cid:` → `cid_cn:` in a markdown document.
/// Touches only the first YAML block at the top of the file; leaves the
/// value, body, and any other keys untouched.
pub fn migrate_cid_to_cid_cn(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return content.to_string(); }
    let leading = &content[..content.len() - trimmed.len()];
    let after = &trimmed[3..];
    let Some(end) = after.find("\n---") else { return content.to_string(); };
    let fm = &after[..end];
    let rest = &after[end..];
    let new_fm: String = fm
        .lines()
        .map(|line| {
            let t = line.trim_start();
            // Root key only (2026-08-01 sweep of the 2026-07-21 trimmed-line class):
            // an INDENTED `cid:` is a user map's child — the `cid_cn` namespace exists
            // precisely so the user's own `cid:` properties are never touched. A root
            // key is unindented, so the rewrite emits at column 0.
            if crate::yaml_lines::is_top_level_key_line(line)
                && t.starts_with("cid:")
                && !t.starts_with("cid_cn")
            {
                format!("cid_cn:{}", &t[4..])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}---{}{}", leading, new_fm, rest)
}

/// Lazy-inject a `cid_cn:` into a note's frontmatter if it doesn't already
/// have one. Uses the file's creation time so the CID's timestamp reflects
/// when the note was originally authored, not when Constellation happened
/// to see it. Called by the note-read pipeline the first time a note is
/// opened in Constellation — no eager vault-wide writes.
///
/// Returns the content (possibly mutated) and writes back to disk only
/// when an injection or migration actually happened.
/// Add `cid_cn: {stem}` to a note's frontmatter, creating the fence when there is none.
///
/// PJ-252 — the newline that follows the opening `---` is STRIPPED before the block is
/// re-emitted, exactly as the siblings `update_frontmatter_title` and `set_frontmatter_parent`
/// already do (PJ-207 §15). The slice after `---` BEGINS at that newline, so re-emitting it
/// under a fresh `---\n` gave the note a blank line above its first property — on the one pass
/// that injects its identity, i.e. the first time the note is ever opened. Confirmed by the
/// 2026-08-11 whole-app safety sweep, then seen on screen in the Boss's own probe note: this was
/// the third sibling of one defect and the only one PJ-207 §15 had not reached.
///
/// Split out of `ensure_cid_cn` so the shape can be tested without touching the disk — the
/// defect lived in string assembly and the enclosing function writes through the gate.
fn inject_cid_cn(content: &str, stem: &str) -> String {
    let trimmed = content.trim_start();
    if trimmed.starts_with("---") {
        let after = &trimmed[3..];
        if let Some(end) = after.find("\n---") {
            let fm_raw = &after[..end];
            let fm = fm_raw
                .strip_prefix("\r\n")
                .or_else(|| fm_raw.strip_prefix('\n'))
                .unwrap_or(fm_raw);
            let body = &after[end + 4..];
            return format!("---\n{}\ncid_cn: {}\n---{}", fm, stem, body);
        }
    }
    format!("---\ncid_cn: {}\n---\n\n{}", stem, content)
}

pub fn ensure_cid_cn(file_path: &Path, content: &str) -> std::io::Result<String> {
    // Already namespaced — nothing to do
    if content.contains("\ncid_cn:") || content.trim_start().starts_with("cid_cn:") {
        return Ok(content.to_string());
    }
    // Legacy key — migrate in place
    if content.contains("\ncid:") || content.trim_start().starts_with("cid:") {
        let migrated = migrate_cid_to_cid_cn(content);
        if migrated != content {
            // MIG-076 §A2 — gated (this runs on the note-OPEN pipeline; the
            // gate serializes it against any concurrent save of the same file).
            crate::write_gate::gate_write(file_path, &migrated, None, "ensure_cid_cn")
                .map_err(std::io::Error::other)?;
        }
        return Ok(migrated);
    }
    // Neither present — synthesise a new CID from the file's creation time
    let created = file_creation_time(file_path);
    let canonical = generate_canonical("NOTE", &created, "md", None);
    let updated = inject_cid_cn(content, &canonical.stem);
    // MIG-076 §A2 — gated.
    crate::write_gate::gate_write(file_path, &updated, None, "ensure_cid_cn")
        .map_err(std::io::Error::other)?;
    Ok(updated)
}

/// Tauri command wrapping `ensure_cid_cn` for call from the frontend's
/// note-open pipeline.
#[tauri::command]
pub fn ensure_cid_cn_cmd(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.is_file() { return Err(format!("Not a file: {}", file_path)); }
    let content = fs::read_to_string(path).map_err(|e| format!("read: {}", e))?;
    ensure_cid_cn(path, &content).map_err(|e| format!("ensure_cid_cn: {}", e))
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
fn library_has_canonical_md(lib_path: &Path, exclude: &std::collections::HashSet<String>) -> bool {
    let files = collect_files_recursive(lib_path, exclude);
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
// App-freeze audit Batch-W (2026-07-04): `(async)` — walks EVERY registered
// library (collect_files_recursive per library) on the boot path; sync it
// parked the dispatch thread for the whole scan. Single fire-and-forget
// caller behind a one-shot localStorage gate (+layout ~2880); writes are
// gate_write/gate_rename per file — safe off-thread.
#[tauri::command(async)]
pub fn repair_external_libraries_on_startup(
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let mut repaired: Vec<String> = Vec::new();
    // PJ-235 — OWN libraries only: this file's three commands RENAME the user's files or
    // revert canonicalization, and the federated resolver would reach into a LINKED
    // universe, which Constellation reads but never writes.
    //
    // Call-graph, VERIFIED 2026-08-11 (an earlier revision of this comment claimed
    // `auto_canonicalize_all` runs at startup — it does not, and the panel review caught
    // the false claim before it entered the record): the startup path is
    // `repair_external_libraries_on_startup` (+layout.svelte:3466, localStorage-gated —
    // and localStorage is proven non-durable, PJ-110/PJ-103, so the gate can reset).
    // `auto_canonicalize_all` is registered (lib.rs) but its only wrapper,
    // `autoCanonicalize()` in importers/store.ts, currently has NO caller in src/.
    let libraries = crate::libraries::load_libraries(&app);
    eprintln!("[CANONICAL] Checking {} libraries for canonical-format files to repair", libraries.len());
    let probe_foreign = crate::libraries::foreign_library_roots(&app, &libraries);
    for lib in &libraries {
        let lib_path = Path::new(&lib.path);
        if !lib_path.is_dir() {
            eprintln!("[CANONICAL]   - {}: path not accessible, skipped", lib.path);
            continue;
        }
        // PJ-235 — same federation-aware boundary as the walk above, so a linked universe's
        // canonical files can never trigger a revert of one of OUR libraries.
        if !library_has_canonical_md(
            lib_path,
            &crate::libraries::walk_exclusions(&libraries, &lib.path, &probe_foreign),
        ) {
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

    /// PJ-252 — injecting the identity must not push a blank line above the note's first
    /// property. `ensure_cid_cn` runs the FIRST time any note is opened, so before this the
    /// blank line was the normal state of every hand-authored note Constellation had ever
    /// opened. Seen on screen in the Boss's Stage-1 probe note, 2026-08-11.
    #[test]
    fn inject_cid_cn_adds_no_blank_line_above_the_first_property() {
        let out = inject_cid_cn("---\ntitle: T\ntags:\n  - a\n---\nbody\n", "STEM");
        assert_eq!(out, "---\ntitle: T\ntags:\n  - a\ncid_cn: STEM\n---\nbody\n");
        assert!(!out.starts_with("---\n\n"), "blank line above the first property: {:?}", out);
    }

    /// The same note as Notepad writes it. The frontmatter's own CRLF endings ride through
    /// untouched; only the fence and the injected line carry the `\n` this writer has always
    /// emitted (shared with `update_frontmatter_title` — not a PJ-252 change, and recorded here
    /// so the next reader knows it was measured rather than overlooked).
    #[test]
    fn inject_cid_cn_crlf_note_gains_no_blank_line() {
        let out = inject_cid_cn("---\r\ntitle: T\r\ntags:\r\n  - a\r\n---\r\nbody\r\n", "STEM");
        assert_eq!(out, "---\ntitle: T\r\ntags:\r\n  - a\r\ncid_cn: STEM\n---\r\nbody\r\n");
        assert!(!out.contains("---\n\r\n"), "blank line above the first property: {:?}", out);
    }

    /// No fence at all, and a fence that never closes, both still synthesise a block.
    #[test]
    fn inject_cid_cn_without_a_usable_fence_synthesises_one() {
        assert_eq!(inject_cid_cn("just body\n", "S"), "---\ncid_cn: S\n---\n\njust body\n");
        assert_eq!(
            inject_cid_cn("---\ntitle: unterminated\n", "S"),
            "---\ncid_cn: S\n---\n\n---\ntitle: unterminated\n"
        );
    }

    /// PJ-182 — an alias appended into a ZERO-INDENT `aliases:` block must join that
    /// block's sequence, not start a deeper one.
    ///
    /// `merge_frontmatter` reads a zero-indent block correctly (it trims before testing
    /// for `- `), but APPENDED with a hardcoded two-space indent. The result still parses
    /// — which is what made it invisible — but `  - "X"` after a column-0 `- Older Name`
    /// is read as a CONTINUATION of that item, so the user's last alias silently becomes
    /// `Older Name - "X"`. Reached by canonicalize / de-canonicalize and by the importer.
    #[test]
    fn pj182_merge_frontmatter_appends_at_the_blocks_own_indent() {
        let existing = "---\ntitle: T\naliases:\n- Old Name\n- Older Name\nstage: seed\n---\nbody";
        let fields = FrontmatterFields {
            title: "T".into(),
            cid: "ABCD".into(),
            kind: "note".into(),
            created: "2026-07-29T00:00:00Z".into(),
            aliases: vec!["Injected".into()],
            original_filename: None,
        };
        let out = merge_frontmatter(existing, &fields);

        // Every item of the aliases sequence shares ONE indentation.
        let mut in_aliases = false;
        let mut indents: Vec<usize> = Vec::new();
        for line in out.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            if t.starts_with("aliases:") {
                in_aliases = true;
                continue;
            }
            if in_aliases {
                if crate::yaml_lines::is_seq_item(line) {
                    indents.push(line.len() - line.trim_start().len());
                } else {
                    in_aliases = false;
                }
            }
        }
        assert!(indents.len() >= 3, "expected the two authored aliases plus ours:\n{out}");
        assert!(
            indents.iter().all(|i| *i == indents[0]),
            "mixed indentation inside one block sequence:\n{out}"
        );
        assert!(out.contains("- Old Name"), "authored alias lost:\n{out}");
        assert!(out.contains("- Older Name"), "authored alias lost:\n{out}");
    }

    /// The control: an INDENTED block still gets an indented append (unchanged behaviour).
    #[test]
    fn pj182_merge_frontmatter_indented_block_is_unchanged() {
        let existing = "---\ntitle: T\naliases:\n  - Old Name\nstage: seed\n---\nbody";
        let fields = FrontmatterFields {
            title: "T".into(),
            cid: "ABCD".into(),
            kind: "note".into(),
            created: "2026-07-29T00:00:00Z".into(),
            aliases: vec!["Injected".into()],
            original_filename: None,
        };
        let out = merge_frontmatter(existing, &fields);
        assert!(out.contains("  - Old Name"), "{out}");
        assert!(out.contains("  - \"Injected\""), "{out}");
    }

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
        assert!(result.contains("cid_cn: 20260410T153045Z_NOTE_7F3A"));
        assert!(!result.contains("\ncid: "), "the legacy key is never emitted (PJ-153)");
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
        assert!(result.contains("cid_cn: 20260410T153045Z_NOTE_ABCD"));
        assert!(!result.contains("\ncid: "), "the legacy key is never emitted (PJ-153)");
        assert!(result.contains("tags:"));
        assert!(result.contains("- rust"));
        assert!(result.contains("Body text."));
    }

    /// PJ-153 (MIG-105 C6) — a legacy `cid:` line in existing frontmatter is
    /// recognized and rewritten as `cid_cn:` with ITS value preserved (the
    /// migrate_cid_to_cid_cn transform): identity survives a re-canonicalize /
    /// re-import; the legacy key never survives (it would index as cid_cn='')
    /// and is never duplicated.
    #[test]
    fn test_merge_migrates_legacy_cid_preserving_value() {
        let content = "---\ntitle: Probe\ncid: 20260704T162439Z_NOTE_1B16\n---\n\nBody text.";
        let fields = FrontmatterFields {
            title: "Probe".to_string(),
            cid: "20260726T000000Z_NOTE_FFFF".to_string(),
            kind: "note".to_string(),
            created: "2026-07-26T00:00:00Z".to_string(),
            aliases: vec![],
            original_filename: None,
        };
        let result = inject_frontmatter(content, &fields);
        assert!(result.contains("cid_cn: 20260704T162439Z_NOTE_1B16"), "value preserved");
        assert!(!result.contains("20260726T000000Z_NOTE_FFFF"), "not re-minted");
        assert!(!result.contains("\ncid: "), "legacy key gone");
        assert_eq!(result.matches("cid_cn:").count(), 1);
        assert!(result.contains("Body text."));
    }

    /// PJ-153 (MIG-105 C6) — an existing `cid_cn:` is preserved untouched and
    /// never duplicated by the add-missing branch.
    #[test]
    fn test_merge_preserves_existing_cid_cn() {
        let content = "---\ntitle: N\ncid_cn: 20260704T162439Z_NOTE_351C\n---\nB";
        let fields = FrontmatterFields {
            title: "N".to_string(),
            cid: "20260726T000000Z_NOTE_EEEE".to_string(),
            kind: "note".to_string(),
            created: "2026-07-26T00:00:00Z".to_string(),
            aliases: vec![],
            original_filename: None,
        };
        let result = inject_frontmatter(content, &fields);
        assert!(result.contains("cid_cn: 20260704T162439Z_NOTE_351C"));
        assert!(!result.contains("20260726T000000Z_NOTE_EEEE"));
        assert_eq!(result.matches("cid_cn:").count(), 1);
    }

    /// PJ-153 (MIG-105 C6) — when BOTH keys exist, `cid_cn:` owns identity and
    /// the legacy line is dropped: the merged block carries exactly one
    /// identity key, regardless of line order.
    #[test]
    fn test_merge_drops_legacy_cid_when_cid_cn_present() {
        let content = "---\ncid: OLDVAL\ncid_cn: 20260704T162439Z_NOTE_351C\ntitle: N\n---\nB";
        let fields = FrontmatterFields {
            title: "N".to_string(),
            cid: "20260726T000000Z_NOTE_DDDD".to_string(),
            kind: "note".to_string(),
            created: "2026-07-26T00:00:00Z".to_string(),
            aliases: vec![],
            original_filename: None,
        };
        let result = inject_frontmatter(content, &fields);
        assert!(result.contains("cid_cn: 20260704T162439Z_NOTE_351C"));
        assert!(!result.contains("OLDVAL"), "the legacy line is dropped, not migrated into a duplicate");
        assert_eq!(result.matches("cid_cn:").count(), 1);
        assert!(!result.contains("\ncid: "));
    }

    /// Shared probe fields for the 2026-08-01 nested-key inspection tests.
    fn probe_fields(cid: &str, aliases: Vec<String>) -> FrontmatterFields {
        FrontmatterFields {
            title: "T".to_string(),
            cid: cid.to_string(),
            kind: "note".to_string(),
            created: "2026-08-01T00:00:00Z".to_string(),
            aliases,
            original_filename: None,
        }
    }

    /// 2026-08-01 inspection (APP-KILLER) — a nested map's `kind:` child is the
    /// USER's data. Matching the trimmed line replaced it with a column-0
    /// `kind: <ours>`, destroying the nested value and the map's structure. The
    /// nested child must survive byte-identically while the note's real root
    /// `kind:` is still minted — and a genuine root `kind:` is still normalized.
    #[test]
    fn merge_preserves_nested_kind_child_and_still_normalizes_root_kind() {
        let content = "---\ntitle: T\nsource:\n  kind: article\n  publisher: X\n---\nBody.";
        let result = inject_frontmatter(content, &probe_fields("20260801T000000Z_NOTE_AAAA", vec![]));
        assert!(
            result.contains("source:\n  kind: article\n  publisher: X"),
            "nested map mangled:\n{result}"
        );
        assert_eq!(
            result.lines().filter(|l| l.starts_with("kind:")).count(),
            1,
            "root kind minted exactly once:\n{result}"
        );
        assert!(result.contains("\nkind: note\n"), "{result}");

        // Control — a genuine ROOT `kind:` is still overwritten with ours.
        let root = inject_frontmatter(
            "---\ntitle: T\nkind: IDEA\n---\nB",
            &probe_fields("20260801T000000Z_NOTE_AAAB", vec![]),
        );
        assert!(!root.contains("IDEA"), "{root}");
        assert_eq!(root.lines().filter(|l| l.starts_with("kind:")).count(), 1, "{root}");
    }

    /// 2026-08-01 inspection (APP-KILLER) — a nested `aliases:` belongs to a user
    /// map: it must not open the alias-injection block, and the injected alias
    /// lands in a fresh ROOT block instead.
    #[test]
    fn merge_does_not_open_alias_block_inside_nested_map() {
        let content = "---\ntitle: T\nsource:\n  aliases:\n    - Foreign One\ntags:\n- t1\n---\nB";
        let result = inject_frontmatter(
            content,
            &probe_fields("20260801T000000Z_NOTE_AAAC", vec!["Injected".to_string()]),
        );
        assert!(
            result.contains("source:\n  aliases:\n    - Foreign One\ntags:\n- t1"),
            "nested map / neighbours mangled:\n{result}"
        );
        assert!(
            result.contains("\naliases:\n  - \"Injected\""),
            "alias not minted at the root:\n{result}"
        );
        assert_eq!(result.matches("Injected").count(), 1, "{result}");
    }

    /// 2026-08-01 inspection (MED) — de-canonicalize strips ONLY the root canonical
    /// fields; a nested `aliases:` (with its items), `kind:`, and
    /// `original_filename:` inside a user map survive byte-identically.
    #[test]
    fn decanonicalize_strips_only_root_canonical_fields() {
        let content = "---\ntitle: T\nsource:\n  aliases:\n    - Keep Me\n  kind: book\n  original_filename: keep.pdf\naliases:\n  - Real Alias\nkind: note\noriginal_filename: \"real.md\"\n---\nB";
        let out = remove_canonical_fields(content);
        assert!(
            out.contains("source:\n  aliases:\n    - Keep Me\n  kind: book\n  original_filename: keep.pdf"),
            "nested user map stripped:\n{out}"
        );
        assert!(!out.lines().any(|l| l.starts_with("aliases:")), "{out}");
        assert!(!out.contains("- Real Alias"), "root alias items not removed:\n{out}");
        assert!(!out.lines().any(|l| l.starts_with("kind:")), "{out}");
        assert!(!out.lines().any(|l| l.starts_with("original_filename:")), "{out}");
    }

    /// 2026-08-01 inspection (APP-KILLER) — a nested `cid_cn:` is a user map's
    /// child, not the note's identity: it must not suppress minting the real root
    /// `cid_cn:`, nor suppress migrating a root legacy `cid:`.
    #[test]
    fn nested_cid_cn_does_not_suppress_root_identity() {
        let content = "---\ntitle: N\nwrapper:\n  cid_cn: NESTED123\n---\nB";
        let result = inject_frontmatter(content, &probe_fields("20260801T000000Z_NOTE_AAAD", vec![]));
        assert!(
            result.contains("wrapper:\n  cid_cn: NESTED123"),
            "nested child mangled:\n{result}"
        );
        assert!(
            result.contains("\ncid_cn: 20260801T000000Z_NOTE_AAAD"),
            "root identity not minted:\n{result}"
        );
        assert_eq!(result.lines().filter(|l| l.starts_with("cid_cn:")).count(), 1, "{result}");

        // A root LEGACY `cid:` still migrates (its value preserved) even with a
        // nested `cid_cn:` present.
        let legacy = inject_frontmatter(
            "---\ntitle: N\ncid: LEGACYVAL\nwrapper:\n  cid_cn: NESTED123\n---\nB",
            &probe_fields("20260801T000000Z_NOTE_AAAE", vec![]),
        );
        assert!(legacy.contains("\ncid_cn: LEGACYVAL"), "legacy identity lost:\n{legacy}");
        assert!(!legacy.contains("\ncid: "), "{legacy}");
        assert!(!legacy.contains("20260801T000000Z_NOTE_AAAE"), "re-minted over legacy:\n{legacy}");
        assert!(legacy.contains("wrapper:\n  cid_cn: NESTED123"), "{legacy}");
        assert_eq!(legacy.lines().filter(|l| l.starts_with("cid_cn:")).count(), 1, "{legacy}");
    }

    /// 2026-08-01 inspection (LOW) — alias dedup is an exact value comparison, not
    /// a substring test: the alias IS appended when its text merely appears inside
    /// the title line, and a genuinely-present alias is still not duplicated.
    /// Covers both append sites (mid-block and end-of-frontmatter).
    #[test]
    fn alias_dedup_is_exact_value_not_substring() {
        let aliases = vec!["My Note".to_string(), "Other".to_string()];
        // Mid-block site: `stage:` closes the alias block.
        let mid = inject_frontmatter(
            "---\ntitle: My Note Extended\naliases:\n  - Other\nstage: seed\n---\nB",
            &probe_fields("20260801T000000Z_NOTE_AAAF", aliases.clone()),
        );
        assert!(
            mid.contains("  - \"My Note\""),
            "absent alias suppressed by substring match against the title:\n{mid}"
        );
        assert!(!mid.contains("- \"Other\""), "present alias duplicated:\n{mid}");
        assert_eq!(mid.matches("- Other").count(), 1, "{mid}");

        // End-of-frontmatter site: the alias block is the last thing in the block.
        let tail = inject_frontmatter(
            "---\ntitle: My Note Extended\naliases:\n  - Other\n---\nB",
            &probe_fields("20260801T000000Z_NOTE_AAAG", aliases),
        );
        assert!(tail.contains("  - \"My Note\""), "{tail}");
        assert!(!tail.contains("- \"Other\""), "{tail}");
    }

    /// 2026-08-01 sweep — `migrate_cid_to_cid_cn` renames the ROOT legacy key only;
    /// a nested `cid:` (a user map's child — the very collision the namespace
    /// exists to avoid) is untouched.
    #[test]
    fn migrate_cid_keeps_nested_cid_child() {
        let out = migrate_cid_to_cid_cn("---\ntitle: T\ncid: ROOTVAL\nref:\n  cid: BOOKID\n---\nB");
        assert!(out.contains("\ncid_cn: ROOTVAL"), "{out}");
        assert!(out.contains("ref:\n  cid: BOOKID"), "nested user cid rewritten:\n{out}");
        assert!(!out.lines().any(|l| l.starts_with("cid:")), "{out}");
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

#[cfg(test)]
mod mig108_slice0_tests {
    use super::*;
    use std::collections::HashSet;

    /// MIG-108 Slice 0 — the canonical file walker must stop at a nested registered
    /// library's boundary. `repair_external_libraries_on_startup` probes EVERY library
    /// with this walker; for the universe_notes library the walk starts at the universe
    /// ROOT, so without the boundary it walked every nested library's files per probe —
    /// and a canonical-repair decision for the ROOT library was being made from OTHER
    /// libraries' files.
    #[test]
    fn canonical_walker_stops_at_a_nested_library_boundary() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("own.md"), "root note").unwrap();

        let nested = root.path().join("Nested Library");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("inner.md"), "nested note").unwrap();

        let norm = |p: &std::path::Path| {
            p.to_string_lossy().replace('\\', "/").trim_end_matches('/').to_lowercase()
        };
        let exclude: HashSet<String> = [norm(&nested)].into_iter().collect();

        let files = collect_files_recursive(root.path(), &exclude);
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"own.md".to_string()));
        assert!(
            !names.contains(&"inner.md".to_string()),
            "a nested registered library's files are its own: {:?}",
            names
        );

        // Control — no exclusion absorbs the nested subtree (the guard is load-bearing).
        let unbounded = collect_files_recursive(root.path(), &HashSet::new());
        assert!(unbounded.iter().any(|p| p.ends_with("inner.md")));
    }
}
