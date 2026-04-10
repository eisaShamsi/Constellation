//! File Kind Registry & Classification Engine
//!
//! Three-layer classifier:
//!   Layer 1: Extension → binary kind (IMG, AUD, VID, ATT, CANVAS, DRAW)
//!   Layer 2: Markdown content heuristics → text kind (NOTE, BASE, TMPL, LINK, MARK, CLIP)
//!   Layer 3: Unknown extension → auto-generate code, persist in registry

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

// ─── Shared constants ────────────────────────────────────────────────

/// Directories to exclude when recursively scanning libraries/vaults.
/// Shared across importers, canonical, and other modules.
pub const EXCLUDED_DIRS: &[&str] = &[
    ".obsidian", ".trash", ".git", ".svn", "node_modules", "__MACOSX", ".constellation",
];

// ─── Core kind codes (compiled into binary) ──────────────────────────

/// All core file kinds that ship with Constellation.
pub const CORE_KINDS: &[(&str, &[&str])] = &[
    // Text-based (classified by content in Layer 2)
    ("NOTE", &[]),  // default for .md — no extension trigger
    ("BASE", &[]),
    ("TMPL", &[]),
    ("LINK", &[]),
    ("MARK", &[]),
    ("CLIP", &[]),
    // Images
    ("IMG", &["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico", "tiff", "tif"]),
    // Audio
    ("AUD", &["mp3", "wav", "ogg", "m4a", "flac", "aac", "wma", "opus"]),
    // Video
    ("VID", &["mp4", "webm", "mov", "mkv", "avi", "wmv", "flv"]),
    // Attachments (documents)
    ("ATT", &["pdf", "docx", "doc", "xlsx", "xls", "pptx", "ppt", "epub",
              "zip", "rar", "7z", "tar", "gz", "rtf", "odt", "ods", "odp"]),
    // Canvas / Drawing
    ("CANVAS", &["canvas"]),
    ("DRAW", &["excalidraw"]),
];

/// Build extension → kind lookup table from CORE_KINDS.
fn build_extension_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (kind, exts) in CORE_KINDS {
        for ext in *exts {
            map.insert(ext.to_lowercase(), kind.to_string());
        }
    }
    map
}

// ─── Custom Kind Registry (persisted per-universe) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomKind {
    pub extensions: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindRegistryFile {
    pub version: u32,
    pub custom_kinds: HashMap<String, CustomKind>,
}

impl Default for KindRegistryFile {
    fn default() -> Self {
        Self {
            version: 1,
            custom_kinds: HashMap::new(),
        }
    }
}

/// The full in-memory kind registry: core + custom.
pub struct KindRegistry {
    ext_map: HashMap<String, String>,    // extension → kind code
    custom: KindRegistryFile,
    config_path: Option<std::path::PathBuf>,
}

impl KindRegistry {
    /// Create a new registry, loading custom kinds from a config path if it exists.
    pub fn new(config_path: Option<&Path>) -> Self {
        let mut ext_map = build_extension_map();
        let custom = if let Some(p) = config_path {
            if p.exists() {
                match fs::read_to_string(p) {
                    Ok(data) => {
                        let reg: KindRegistryFile =
                            serde_json::from_str(&data).unwrap_or_default();
                        // Merge custom extensions into the lookup
                        for (code, kind) in &reg.custom_kinds {
                            for ext in &kind.extensions {
                                let e = ext.trim_start_matches('.').to_lowercase();
                                ext_map.insert(e, code.clone());
                            }
                        }
                        reg
                    }
                    Err(_) => KindRegistryFile::default(),
                }
            } else {
                KindRegistryFile::default()
            }
        } else {
            KindRegistryFile::default()
        };

        Self {
            ext_map,
            custom,
            config_path: config_path.map(|p| p.to_path_buf()),
        }
    }

    /// Look up kind code by file extension.
    pub fn kind_by_extension(&self, ext: &str) -> Option<&str> {
        self.ext_map.get(&ext.to_lowercase()).map(|s| s.as_str())
    }

    /// Check if a kind code exists (core or custom).
    pub fn has_code(&self, code: &str) -> bool {
        let upper = code.to_uppercase();
        CORE_KINDS.iter().any(|(k, _)| *k == upper) || self.custom.custom_kinds.contains_key(&upper)
    }

    /// Auto-generate a kind code for an unknown extension.
    /// Registers it in custom_kinds and persists.
    pub fn auto_generate(&mut self, extension: &str) -> String {
        let clean = extension.trim_start_matches('.').to_uppercase();
        let base = if clean.len() > 6 {
            clean[..6].to_string()
        } else {
            clean.clone()
        };

        // Find a non-colliding code
        let code = if !self.has_code(&base) {
            base.clone()
        } else {
            let mut candidate = base.clone();
            for i in 1..=9 {
                candidate = format!("{}{}", base, i);
                if !self.has_code(&candidate) {
                    break;
                }
            }
            candidate
        };

        // Register
        self.ext_map.insert(
            extension.trim_start_matches('.').to_lowercase(),
            code.clone(),
        );
        self.custom.custom_kinds.insert(
            code.clone(),
            CustomKind {
                extensions: vec![format!(
                    ".{}",
                    extension.trim_start_matches('.').to_lowercase()
                )],
                description: String::new(),
            },
        );

        // Persist
        self.save();
        code
    }

    /// Save the custom kinds to disk.
    fn save(&self) {
        if let Some(ref path) = self.config_path {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&self.custom) {
                let _ = fs::write(path, json);
            }
        }
    }
}

// ─── Classification Engine ───────────────────────────────────────────

/// Classify a file by its path and content.
/// Returns the kind code (e.g., "NOTE", "IMG", "TMPL").
pub fn classify_file(file_path: &Path, registry: &mut KindRegistry) -> String {
    let ext = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Layer 1: Extension-based (binary types)
    if let Some(kind) = registry.kind_by_extension(&ext) {
        // For .md files, fall through to Layer 2
        if ext != "md" && ext != "markdown" {
            return kind.to_string();
        }
    }

    // Layer 2: Markdown content heuristics
    if ext == "md" || ext == "markdown" {
        return classify_markdown(file_path);
    }

    // .json could be canvas data
    if ext == "json" {
        if let Ok(content) = read_head(file_path, 4096) {
            if content.contains("\"nodes\"") && content.contains("\"edges\"") {
                return "CANVAS".to_string();
            }
        }
        return "ATT".to_string();
    }

    // Layer 3: Unknown extension → auto-generate
    if ext.is_empty() {
        return "ATT".to_string();
    }
    registry.auto_generate(&ext)
}

/// Classify a markdown file by analyzing its frontmatter and content structure.
fn classify_markdown(file_path: &Path) -> String {
    let content = match read_head(file_path, 4096) {
        Ok(c) => c,
        Err(_) => return "NOTE".to_string(),
    };

    let (frontmatter, body) = parse_frontmatter_raw(&content);

    // Priority 1: Explicit type/kind field in frontmatter
    if let Some(ref fm) = frontmatter {
        let fm_lower = fm.to_lowercase();
        if let Some(kind) = extract_yaml_value(&fm_lower, "kind")
            .or_else(|| extract_yaml_value(&fm_lower, "type"))
        {
            match kind.trim() {
                "note" | "conote" => return "NOTE".to_string(),
                "base" | "database" | "db" => return "BASE".to_string(),
                "template" | "tmpl" => return "TMPL".to_string(),
                "link" | "connection" => return "LINK".to_string(),
                "bookmark" | "mark" => return "MARK".to_string(),
                "clip" | "clipping" | "webclip" => return "CLIP".to_string(),
                "canvas" => return "CANVAS".to_string(),
                _ => {} // Unknown type value, continue heuristics
            }
        }
    }

    // Priority 2: LINK — has from: AND to: fields
    if let Some(ref fm) = frontmatter {
        let has_from = fm.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("from:") || t.starts_with("from :")
        });
        let has_to = fm.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("to:") || t.starts_with("to :")
        });
        if has_from && has_to {
            return "LINK".to_string();
        }
    }

    // Priority 3: TMPL — template syntax
    if content.contains("<%") && content.contains("%>") {
        return "TMPL".to_string();
    }
    if content.contains("{{") && content.contains("}}") {
        // Could be Handlebars/Mustache template — but also valid in some markdown
        // Only classify as template if multiple occurrences or frontmatter hints
        let template_count = content.matches("{{").count();
        if template_count >= 3 {
            return "TMPL".to_string();
        }
    }
    if let Some(ref fm) = frontmatter {
        if fm.contains("template:") && fm.contains("true") {
            return "TMPL".to_string();
        }
    }

    // Priority 4: MARK — bookmark (url: field, short body)
    if let Some(ref fm) = frontmatter {
        let has_url = fm.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("url:") || t.starts_with("bookmark:")
        });
        if has_url && body.len() < 500 {
            return "MARK".to_string();
        }
    }

    // Priority 5: CLIP — source: field + blockquotes
    if let Some(ref fm) = frontmatter {
        let has_source = fm.lines().any(|l| l.trim_start().starts_with("source:"));
        let has_blockquote = body.lines().any(|l| l.trim_start().starts_with('>'));
        if has_source && has_blockquote {
            return "CLIP".to_string();
        }
    }

    // Priority 6: BASE — database structure
    if let Some(ref fm) = frontmatter {
        let has_schema = fm.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("schema:") || t.starts_with("fields:") || t.starts_with("database:")
        });
        if has_schema {
            return "BASE".to_string();
        }
    }
    // Dataview blocks in body
    if body.contains("```dataview") || body.contains("```dataviewjs") {
        return "BASE".to_string();
    }

    // Priority 7: CANVAS — JSON with nodes/edges (rare for .md, but possible)
    if body.trim_start().starts_with('{') {
        if body.contains("\"nodes\"") && body.contains("\"edges\"") {
            return "CANVAS".to_string();
        }
    }

    // Default
    "NOTE".to_string()
}

// ─── Helpers ─────────────────────────────────────────────────────────

/// Read the first `max_bytes` of a file as a UTF-8 string.
/// Handles UTF-8 boundary safely.
fn read_head(path: &Path, max_bytes: usize) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf)?;
    buf.truncate(n);
    // Walk backward to valid UTF-8 boundary
    let s = match std::str::from_utf8(&buf) {
        Ok(s) => s.to_string(),
        Err(e) => {
            let valid_up_to = e.valid_up_to();
            String::from_utf8_lossy(&buf[..valid_up_to]).to_string()
        }
    };
    Ok(s)
}

/// Split content into (frontmatter, body). Frontmatter is the YAML between --- delimiters.
fn parse_frontmatter_raw(content: &str) -> (Option<String>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }
    // Find the closing ---
    let after_first = &trimmed[3..];
    if let Some(end) = after_first.find("\n---") {
        let fm = &after_first[..end];
        let body_start = end + 4; // skip \n---
        let body = if body_start < after_first.len() {
            &after_first[body_start..]
        } else {
            ""
        };
        (Some(fm.to_string()), body)
    } else {
        (None, content)
    }
}

/// Extract a simple `key: value` from YAML frontmatter (single-line values only).
fn extract_yaml_value<'a>(yaml: &'a str, key: &str) -> Option<String> {
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(key) {
            let rest = &trimmed[key.len()..];
            if rest.starts_with(':') {
                let val = rest[1..].trim();
                // Strip quotes
                let val = val.trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

// ─── Tauri Commands ──────────────────────────────────────────────────

#[tauri::command]
pub fn classify_file_cmd(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }
    // Use a temporary registry without persistence for one-off classification
    let mut registry = KindRegistry::new(None);
    Ok(classify_file(file_path, &mut registry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_map() {
        let registry = KindRegistry::new(None);
        assert_eq!(registry.kind_by_extension("png"), Some("IMG"));
        assert_eq!(registry.kind_by_extension("PNG"), Some("IMG"));
        assert_eq!(registry.kind_by_extension("mp3"), Some("AUD"));
        assert_eq!(registry.kind_by_extension("pdf"), Some("ATT"));
        assert_eq!(registry.kind_by_extension("canvas"), Some("CANVAS"));
        assert_eq!(registry.kind_by_extension("xyz"), None);
    }

    #[test]
    fn test_auto_generate() {
        let mut registry = KindRegistry::new(None);
        let code = registry.auto_generate("blend");
        assert_eq!(code, "BLEND");
        assert_eq!(registry.kind_by_extension("blend"), Some("BLEND"));
    }

    #[test]
    fn test_frontmatter_parsing() {
        let content = "---\ntitle: Test\ntype: template\n---\nBody here";
        let (fm, body) = parse_frontmatter_raw(content);
        assert!(fm.is_some());
        assert!(body.contains("Body here"));

        let fm_str = fm.unwrap();
        let val = extract_yaml_value(&fm_str, "type");
        assert_eq!(val, Some("template".to_string()));
    }
}
