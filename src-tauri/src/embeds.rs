// Living Embed Resolver — universal file resolution for embed-like links
// (`![[target]]`, `![[subfolder/target.ext]]`, `![[note#heading]]`).
//
// Designed to match Obsidian's resolution rules precisely, then extend them
// for other PKM imports (LogSeq, Roam, Notion) via the same unified pipeline.
//
// Resolution order:
//   1. Exact path relative to the note's folder
//   2. Exact absolute path inside the vault
//   3. Explicit attachment folder from `.obsidian/app.json`
//   4. Common attachment folder fallbacks (attachments/, images/, assets/)
//   5. Vault-wide filename index (Obsidian's "shortest unambiguous" rule)
//   6. Vault root
//
// Returns EmbedResolution with a typed `kind` so the frontend can choose
// the appropriate widget (image, audio, video, PDF, canvas, excalidraw,
// note transclusion, generic file, or missing).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Kinds of embed content. Frontend picks a widget by kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedKind {
    Image,
    Audio,
    Video,
    Pdf,
    Canvas,       // Obsidian .canvas JSON
    Excalidraw,   // .excalidraw JSON
    Note,         // markdown — transclusion target
    Generic,      // any other file type
    Missing,      // target not found in vault
}

/// Resolution result for an embed target.
#[derive(Debug, Clone, Serialize)]
pub struct EmbedResolution {
    pub kind: String,                 // EmbedKind as snake_case
    pub url: String,                  // asset URL the frontend can use directly
    pub absolute_path: Option<String>,
    pub mime: Option<String>,
    pub size_bytes: u64,
    /// For transclusions: raw markdown body (caller can scope to heading/block).
    pub note_body: Option<String>,
    pub heading: Option<String>,      // `#Heading` fragment if present
    pub block_id: Option<String>,     // `^block-id` fragment if present
}

impl EmbedResolution {
    pub fn missing() -> Self {
        Self {
            kind: "missing".into(),
            url: String::new(),
            absolute_path: None,
            mime: None,
            size_bytes: 0,
            note_body: None,
            heading: None,
            block_id: None,
        }
    }
}

/// Parsed subset of Obsidian's `.obsidian/app.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VaultConfig {
    /// "", "./", "folder/path" — where Obsidian stores new attachments.
    #[serde(default)]
    pub attachment_folder_path: String,
    #[serde(default)]
    pub use_markdown_links: bool,
    /// "shortest" | "relative" | "absolute"
    #[serde(default = "default_new_link_format")]
    pub new_link_format: String,
}

fn default_new_link_format() -> String { "shortest".into() }

/// Read `.obsidian/app.json` from a vault and extract the fields we care about.
/// Missing file / bad JSON are non-fatal — defaults apply.
pub fn read_vault_config(library_path: &Path) -> VaultConfig {
    let cfg_path = library_path.join(".obsidian").join("app.json");
    let Ok(raw) = fs::read_to_string(&cfg_path) else { return VaultConfig::default(); };
    // Obsidian's schema is loose; use a permissive interim struct
    #[derive(Deserialize)]
    struct Raw {
        #[serde(default, rename = "attachmentFolderPath")]
        attachment_folder_path: String,
        #[serde(default, rename = "useMarkdownLinks")]
        use_markdown_links: bool,
        #[serde(default, rename = "newLinkFormat")]
        new_link_format: Option<String>,
    }
    let Ok(r) = serde_json::from_str::<Raw>(&raw) else { return VaultConfig::default(); };
    VaultConfig {
        attachment_folder_path: r.attachment_folder_path,
        use_markdown_links: r.use_markdown_links,
        new_link_format: r.new_link_format.unwrap_or_else(default_new_link_format),
    }
}

// ─── Vault filename index ──────────────────────────────────────────
//
// One-time walkdir over the vault produces `Map<lowercaseFilename, path>`.
// For duplicate filenames the first occurrence wins (Obsidian's "shortest
// unambiguous" rule — we approximate by walking depth-first and keeping the
// shallowest hit). Index is cached per library path with an invalidation
// counter that the filesystem watcher bumps on change.

pub struct VaultIndex {
    pub files: HashMap<String, PathBuf>, // lowercase basename → absolute path
    pub built_at: std::time::Instant,
}

fn vault_index_cache() -> &'static Mutex<HashMap<String, VaultIndex>> {
    static CELL: OnceLock<Mutex<HashMap<String, VaultIndex>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

const IGNORED_DIRS: &[&str] = &[".git", ".obsidian", ".trash", "node_modules", ".DS_Store"];

pub fn build_vault_index(library_path: &Path) -> VaultIndex {
    let mut files: HashMap<String, PathBuf> = HashMap::new();
    walk(library_path, &mut |path, depth| {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            let key = name.to_lowercase();
            // Keep the shallowest hit for duplicates
            files.entry(key).or_insert_with(|| path.to_path_buf());
            let _ = depth;
        }
    });
    VaultIndex { files, built_at: std::time::Instant::now() }
}

fn walk<F: FnMut(&Path, usize)>(root: &Path, callback: &mut F) {
    fn inner<F: FnMut(&Path, usize)>(dir: &Path, depth: usize, cb: &mut F) {
        let Ok(entries) = fs::read_dir(dir) else { return; };
        for e in entries.flatten() {
            let path = e.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if IGNORED_DIRS.contains(&name) { continue; }
            }
            let md = match e.metadata() { Ok(m) => m, Err(_) => continue };
            if md.is_dir() {
                inner(&path, depth + 1, cb);
            } else if md.is_file() {
                cb(&path, depth);
            }
        }
    }
    inner(root, 0, callback);
}

pub fn get_or_build_vault_index(library_path: &str) -> HashMap<String, PathBuf> {
    if let Ok(c) = vault_index_cache().lock() {
        if let Some(idx) = c.get(library_path) {
            return idx.files.clone();
        }
    }
    let idx = build_vault_index(Path::new(library_path));
    let files = idx.files.clone();
    if let Ok(mut c) = vault_index_cache().lock() {
        c.insert(library_path.into(), idx);
    }
    files
}

pub fn invalidate_vault_index(library_path: &str) {
    if let Ok(mut c) = vault_index_cache().lock() {
        c.remove(library_path);
    }
}

// ─── Target parsing ─────────────────────────────────────────────────

/// Split `target` (everything inside `![[...]]`) into filename + optional
/// heading / block-id / query fragments.
pub struct ParsedTarget {
    pub path: String,         // filename or relative path, minus fragments
    pub heading: Option<String>,
    pub block_id: Option<String>,
    pub query: Option<String>, // e.g. `#page=3` for PDFs
}

pub fn parse_target(raw: &str) -> ParsedTarget {
    let raw = raw.trim();
    // Strip any display alias `![[target|alias]]` — the alias doesn't affect resolution
    let raw = raw.split('|').next().unwrap_or(raw);
    // `#page=N` style query for PDFs (Obsidian uses this too)
    let (main, query) = if let Some(i) = raw.find("#page=") {
        (&raw[..i], Some(raw[i+1..].to_string()))
    } else { (raw, None) };
    // Heading / block-id — # for heading, ^ for block
    if let Some(i) = main.find('^') {
        return ParsedTarget {
            path: main[..i].to_string(),
            heading: None,
            block_id: Some(main[i+1..].to_string()),
            query,
        };
    }
    if let Some(i) = main.find('#') {
        return ParsedTarget {
            path: main[..i].to_string(),
            heading: Some(main[i+1..].to_string()),
            block_id: None,
            query,
        };
    }
    ParsedTarget { path: main.to_string(), heading: None, block_id: None, query }
}

// ─── Resolution ─────────────────────────────────────────────────────

fn classify(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "avif" | "ico" => "image",
        "mp3" | "wav" | "ogg" | "oga" | "m4a" | "flac" | "3gp" | "opus" => "audio",
        "mp4" | "webm" | "mov" | "mkv" | "ogv" | "m4v" => "video",
        "pdf" => "pdf",
        "canvas" => "canvas",
        "excalidraw" => "excalidraw",
        "md" | "markdown" | "mdx" => "note",
        _ => "generic",
    }
}

fn mime_for(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" | "oga" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "opus" => "audio/opus",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",
        "ogv" => "video/ogg",
        "pdf" => "application/pdf",
        "canvas" | "excalidraw" | "json" => "application/json",
        "md" | "markdown" | "mdx" => "text/markdown",
        _ => "application/octet-stream",
    }
}

/// Size threshold above which we return an `asset://` URL instead of embedding
/// the file bytes as a data URL. Keeps the WebView off the GC path for big
/// videos / PDFs while tiny thumbnails inline cleanly.
const INLINE_MAX_BYTES: u64 = 4 * 1024 * 1024; // 4 MB

fn build_asset_url(path: &Path) -> String {
    // Tauri's `asset://` protocol is enabled in tauri.conf.json. Host rewrite
    // handles per-OS path quirks.
    let mut s = path.to_string_lossy().to_string();
    #[cfg(windows)]
    { s = s.replace('\\', "/"); }
    format!("asset://localhost/{}", url_encode_path(&s))
}

/// Minimal URL-encoder for filesystem path components. Reserves letters,
/// digits, `-`, `_`, `.`, `~`, and forward slash (Tauri routes expect literal
/// path separators). Everything else is percent-encoded as UTF-8 bytes.
fn url_encode_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || "-_.~/:".contains(ch) {
            out.push(ch);
        } else {
            let mut buf = [0u8; 4];
            for b in ch.encode_utf8(&mut buf).as_bytes() {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

fn encode_data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{};base64,{}", mime, base64_encode(bytes))
}

/// Hand-rolled base64 encoder — mirrors the one in libraries.rs so embeds.rs
/// stays self-contained without new crate dependencies.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(TABLE[((n >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(TABLE[(n & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

fn resolve_path(
    library_path: &str,
    note_path: &str,
    target: &str,
    cfg: &VaultConfig,
) -> Option<PathBuf> {
    let sep = std::path::MAIN_SEPARATOR;
    let lib = Path::new(library_path);

    // 1. Relative to note's folder (exact path)
    if !note_path.is_empty() {
        if let Some(note_dir) = Path::new(note_path).parent() {
            let p = note_dir.join(target);
            if p.is_file() { return Some(p); }
        }
    }

    // 2. Absolute inside vault (`![[subfolder/file.png]]`)
    if target.contains('/') || target.contains('\\') {
        let p = lib.join(target);
        if p.is_file() { return Some(p); }
    }

    // 3. Explicit attachment folder from .obsidian/app.json
    let attach = &cfg.attachment_folder_path;
    if !attach.is_empty() {
        // Obsidian accepts "./" (same folder as note), "./subfolder" (relative),
        // "folder" (absolute in vault).
        let p = if attach == "./" || attach.is_empty() {
            Path::new(note_path).parent().map(|p| p.join(target)).unwrap_or_else(|| lib.join(target))
        } else if attach.starts_with("./") {
            Path::new(note_path).parent()
                .map(|p| p.join(&attach[2..]).join(target))
                .unwrap_or_else(|| lib.join(&attach[2..]).join(target))
        } else {
            lib.join(attach).join(target)
        };
        if p.is_file() { return Some(p); }
    }

    // 4. Common attachment folder fallbacks
    for folder in &["attachments", "images", "assets", "media", "_attachments", "Attachments"] {
        let p = lib.join(folder).join(target);
        if p.is_file() { return Some(p); }
    }

    // 5. Vault-wide filename index (Obsidian's default behavior)
    // Only applies if target has no path separator — path-form is already handled above.
    if !target.contains('/') && !target.contains('\\') {
        let key = target.to_lowercase();
        let index = get_or_build_vault_index(library_path);
        if let Some(p) = index.get(&key) {
            if p.is_file() { return Some(p.clone()); }
        }
        // Markdown transclusion: user may have typed `![[Note]]` without .md
        if Path::new(target).extension().is_none() {
            let key_md = format!("{}.md", key);
            if let Some(p) = index.get(&key_md) {
                if p.is_file() { return Some(p.clone()); }
            }
        }
    }

    // 6. Vault root
    let p = lib.join(target);
    if p.is_file() { return Some(p); }
    let _ = sep;
    None
}

/// Main entry point. Resolves an embed target to a URL the frontend can render.
#[tauri::command]
pub fn resolve_embed(
    library_path: String,
    note_path: String,
    target: String,
) -> EmbedResolution {
    let parsed = parse_target(&target);
    let cfg = read_vault_config(Path::new(&library_path));

    let Some(abs) = resolve_path(&library_path, &note_path, &parsed.path, &cfg) else {
        return EmbedResolution {
            kind: "missing".into(),
            url: String::new(),
            absolute_path: None,
            mime: None,
            size_bytes: 0,
            note_body: None,
            heading: parsed.heading,
            block_id: parsed.block_id,
        };
    };

    let ext = abs.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let kind = classify(&ext);
    let mime = mime_for(&ext);
    let size = fs::metadata(&abs).map(|m| m.len()).unwrap_or(0);

    // Notes: read body; caller will scope to heading / block-id in JS
    if kind == "note" {
        let body = fs::read_to_string(&abs).unwrap_or_default();
        return EmbedResolution {
            kind: "note".into(),
            url: String::new(),
            absolute_path: Some(abs.to_string_lossy().into_owned()),
            mime: Some(mime.into()),
            size_bytes: size,
            note_body: Some(body),
            heading: parsed.heading,
            block_id: parsed.block_id,
        };
    }

    // Canvas / Excalidraw: return the raw JSON body so the frontend can parse
    // and render natively.
    if kind == "canvas" || kind == "excalidraw" {
        let body = fs::read_to_string(&abs).unwrap_or_default();
        return EmbedResolution {
            kind: kind.into(),
            url: build_asset_url(&abs),
            absolute_path: Some(abs.to_string_lossy().into_owned()),
            mime: Some(mime.into()),
            size_bytes: size,
            note_body: Some(body),
            heading: parsed.heading,
            block_id: parsed.block_id,
        };
    }

    // Small binaries: inline as data URL (fast, no extra IPC for asset fetch).
    // Large binaries: return asset://localhost URL so the WebView streams them.
    let url = if size <= INLINE_MAX_BYTES {
        if let Ok(bytes) = fs::read(&abs) {
            encode_data_url(&bytes, mime)
        } else {
            build_asset_url(&abs)
        }
    } else {
        build_asset_url(&abs)
    };

    EmbedResolution {
        kind: kind.into(),
        url,
        absolute_path: Some(abs.to_string_lossy().into_owned()),
        mime: Some(mime.into()),
        size_bytes: size,
        note_body: None,
        heading: parsed.heading,
        block_id: parsed.block_id,
    }
}

/// Expose the vault config so the frontend can show "Attachment folder: ..." in library settings.
#[tauri::command]
pub fn read_vault_config_cmd(library_path: String) -> VaultConfig {
    read_vault_config(Path::new(&library_path))
}

/// Frontend can call this after large vault mutations to refresh the index.
#[tauri::command]
pub fn invalidate_vault_index_cmd(library_path: String) {
    invalidate_vault_index(&library_path);
}
