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
    /// Diagnostic info when kind == "missing": list of candidate paths we tried
    /// and the vault's configured attachment folder. Helps the UI tell the user
    /// WHY the file wasn't found.
    #[serde(default)]
    pub tried_paths: Vec<String>,
    #[serde(default)]
    pub attachment_folder: String,
    /// Files in the vault whose name is close to the target — "did you mean"
    /// suggestions for a missing resolution.
    #[serde(default)]
    pub similar_files: Vec<String>,
    /// Total file count in the vault index — when this is 0, the walk itself
    /// failed (permission denied at vault root, wrong path, etc.) and no
    /// individual path comparison would ever succeed.
    #[serde(default)]
    pub vault_file_count: u64,
    /// Direct filesystem listing of the expected attachment folder, if it
    /// exists. Bypasses the index cache — shows the OS's actual view of
    /// the folder right now, which is definitive proof that a file is or
    /// isn't there.
    #[serde(default)]
    pub attachment_folder_listing: Vec<String>,
    #[serde(default)]
    pub attachment_folder_resolved: String,
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
            tried_paths: Vec::new(),
            attachment_folder: String::new(),
            similar_files: Vec::new(),
            vault_file_count: 0,
            attachment_folder_listing: Vec::new(),
            attachment_folder_resolved: String::new(),
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

/// MIG-108 Slice 0 — `exclude` = `nested_library_paths`: a subdirectory that is itself a
/// REGISTERED LIBRARY is skipped ("Library ≠ Folder"). Without it, the universe_notes
/// library's index (path == universe root) absorbed every nested library's files, so a
/// nested file could shadow same-named files via first-wins `or_insert` — and post-MIG-108
/// every library is nested under the root.
pub fn build_vault_index(library_path: &Path, exclude: &std::collections::HashSet<String>) -> VaultIndex {
    let mut files: HashMap<String, PathBuf> = HashMap::new();
    walk(library_path, exclude, &mut |path, depth| {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            // Primary key: lowercased original name
            files.entry(name.to_lowercase()).or_insert_with(|| path.to_path_buf());
            // Secondary key: digit-normalized (Arabic-Indic / Persian / ASCII
            // all collapse to the same form so `![[Pasted image ٢٠٢٥.png]]`
            // resolves even when disk has `Pasted image 2025.png`, and vice
            // versa. Common with Obsidian on Arabic/Persian locales.
            let normalized = normalize_digits(&name.to_lowercase());
            if normalized != name.to_lowercase() {
                files.entry(normalized).or_insert_with(|| path.to_path_buf());
            }
            let _ = depth;
        }
    });
    VaultIndex { files, built_at: std::time::Instant::now() }
}

/// Fold Arabic-Indic (U+0660..0669) and Extended Arabic-Indic / Persian
/// (U+06F0..06F9) digits down to ASCII 0-9. Leaves all other characters
/// untouched. Used to normalize filenames whose digit encoding differs from
/// what the note references.
pub fn normalize_digits(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        let cp = ch as u32;
        if (0x0660..=0x0669).contains(&cp) {
            // Arabic-Indic 0-9
            out.push(char::from_u32('0' as u32 + (cp - 0x0660)).unwrap_or(ch));
        } else if (0x06F0..=0x06F9).contains(&cp) {
            // Extended Arabic-Indic / Persian / Urdu 0-9
            out.push(char::from_u32('0' as u32 + (cp - 0x06F0)).unwrap_or(ch));
        } else {
            out.push(ch);
        }
    }
    out
}

fn walk<F: FnMut(&Path, usize)>(root: &Path, exclude: &std::collections::HashSet<String>, callback: &mut F) {
    fn inner<F: FnMut(&Path, usize)>(
        dir: &Path,
        depth: usize,
        exclude: &std::collections::HashSet<String>,
        cb: &mut F,
    ) {
        let Ok(entries) = fs::read_dir(dir) else { return; };
        for e in entries.flatten() {
            let path = e.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if IGNORED_DIRS.contains(&name) { continue; }
            }
            let md = match e.metadata() { Ok(m) => m, Err(_) => continue };
            if md.is_dir() {
                if crate::libraries::is_walk_boundary(&path, exclude) { continue; } // a library or a universe is never content (MIG-112)
                inner(&path, depth + 1, exclude, cb);
            } else if md.is_file() {
                cb(&path, depth);
            }
        }
    }
    inner(root, 0, exclude, callback);
}

// Batch-W (2026-07-04): single-flight guard for the cold build. With
// resolve_embed `(async)`, N embeds mounting cold would otherwise run N
// CONCURRENT full-library walks (the sync dispatch thread used to serialize
// them into 1 walk + N−1 cache hits). Held only across the cold build, on
// Tokio blocking threads — never on the dispatch thread.
static BUILD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn get_or_build_vault_index(library_path: &str, exclude: &std::collections::HashSet<String>) -> HashMap<String, PathBuf> {
    if let Ok(c) = vault_index_cache().lock() {
        if let Some(idx) = c.get(library_path) {
            return idx.files.clone();
        }
    }
    let _build_guard = BUILD_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // Double-check: another builder may have populated the cache while we
    // waited for the lock.
    if let Ok(c) = vault_index_cache().lock() {
        if let Some(idx) = c.get(library_path) {
            return idx.files.clone();
        }
    }
    let idx = build_vault_index(Path::new(library_path), exclude);
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

/// MIG-108 Slice 0 — clear EVERY cached index. Called whenever the library REGISTRY
/// changes (save_libraries): registering or removing a library changes which subtrees the
/// parent library's index must exclude, and a stale pre-registration index would keep
/// serving the now-foreign files until its next unrelated invalidation.
pub fn invalidate_all_vault_indexes() {
    if let Ok(mut c) = vault_index_cache().lock() {
        c.clear();
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

/// Result of a resolution attempt: the matched path (if any) and the list
/// of paths we tried on the way. `tried` is useful for "file not found"
/// diagnostics shown to the user.
struct ResolutionResult {
    matched: Option<PathBuf>,
    tried: Vec<PathBuf>,
}

fn resolve_path(
    library_path: &str,
    note_path: &str,
    target: &str,
    cfg: &VaultConfig,
    exclude: &std::collections::HashSet<String>,
) -> ResolutionResult {
    let lib = Path::new(library_path);
    let mut tried: Vec<PathBuf> = Vec::new();

    // Try the target both as-provided and with all digit codepoints folded to
    // ASCII. Handles Obsidian's Arabic/Persian-locale behavior where pasted
    // images may be saved with ٠-٩ or ۰-۹ in the filename while a note
    // references the ASCII form (or vice versa).
    let target_normalized = normalize_digits(target);
    let target_variants: Vec<String> = if target_normalized != target {
        vec![target.to_string(), target_normalized]
    } else {
        vec![target.to_string()]
    };

    let mut try_candidate = |base: PathBuf, tried: &mut Vec<PathBuf>| -> Option<PathBuf> {
        for variant in &target_variants {
            let p = base.join(variant);
            if p.is_file() { return Some(p); }
            if !tried.iter().any(|t| t == &p) { tried.push(p); }
        }
        None
    };

    // 1. Relative to note's folder (exact path)
    if !note_path.is_empty() {
        if let Some(note_dir) = Path::new(note_path).parent() {
            if let Some(p) = try_candidate(note_dir.to_path_buf(), &mut tried) {
                return ResolutionResult { matched: Some(p), tried };
            }
        }
    }

    // 2. Absolute inside vault (`![[subfolder/file.png]]`)
    if target.contains('/') || target.contains('\\') {
        if let Some(p) = try_candidate(lib.to_path_buf(), &mut tried) {
            return ResolutionResult { matched: Some(p), tried };
        }
    }

    // 3. Explicit attachment folder from .obsidian/app.json
    let attach = &cfg.attachment_folder_path;
    if !attach.is_empty() {
        let base = if attach == "./" {
            Path::new(note_path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| lib.to_path_buf())
        } else if attach.starts_with("./") {
            Path::new(note_path).parent()
                .map(|p| p.join(&attach[2..]))
                .unwrap_or_else(|| lib.join(&attach[2..]))
        } else {
            lib.join(attach)
        };
        if let Some(p) = try_candidate(base, &mut tried) {
            return ResolutionResult { matched: Some(p), tried };
        }
    }

    // 4. Common attachment folder fallbacks
    for folder in &["attachments", "images", "assets", "media", "_attachments", "Attachments", "Files", "files"] {
        if let Some(p) = try_candidate(lib.join(folder), &mut tried) {
            return ResolutionResult { matched: Some(p), tried };
        }
    }

    // 5. Vault-wide filename index (Obsidian's default — finds the file regardless
    //    of how deeply it's nested). Rebuild on miss so newly-added files are seen.
    if !target.contains('/') && !target.contains('\\') {
        if let Some(p) = lookup_in_library_index(library_path, target, exclude) {
            return ResolutionResult { matched: Some(p), tried };
        }
        // Miss — force a fresh index and retry (handles files added since last scan)
        invalidate_vault_index(library_path);
        if let Some(p) = lookup_in_library_index(library_path, target, exclude) {
            return ResolutionResult { matched: Some(p), tried };
        }
    }

    // 6. Vault root
    if let Some(p) = try_candidate(lib.to_path_buf(), &mut tried) {
        return ResolutionResult { matched: Some(p), tried };
    }

    ResolutionResult { matched: None, tried }
}

/// Look a bare `![[target]]` filename up in one library's cached index —
/// both the original lowercased key AND the digit-normalized key (the index
/// stores both forms for names containing non-ASCII digits), with the `.md`
/// fallback for extensionless note transclusions. Cache-through: builds the
/// index on first use (BUILD_LOCK single-flight).
fn lookup_in_library_index(library_path: &str, target: &str, exclude: &std::collections::HashSet<String>) -> Option<PathBuf> {
    let key = target.to_lowercase();
    let normalized_key = normalize_digits(&key);
    let try_keys: Vec<&str> = if normalized_key != key { vec![&key, &normalized_key] } else { vec![&key] };
    let index = get_or_build_vault_index(library_path, exclude);
    for k in &try_keys {
        if let Some(p) = index.get(*k) { if p.is_file() { return Some(p.clone()); } }
    }
    // Transclusion without .md extension
    if Path::new(target).extension().is_none() {
        for k in &try_keys {
            let k_md = format!("{}.md", k);
            if let Some(p) = index.get(&k_md) { if p.is_file() { return Some(p.clone()); } }
        }
    }
    None
}

/// Main entry point. Resolves an embed target to a URL the frontend can render.
// App-freeze audit Batch-W (2026-07-04): `(async)` — a cold/invalidated call
// walks the whole library (get_or_build_vault_index); BUILD_LOCK above keeps
// concurrent cold embeds to one walk.
#[tauri::command(async)]
pub fn resolve_embed(
    app: tauri::AppHandle,
    library_path: String,
    note_path: String,
    target: String,
) -> EmbedResolution {
    let parsed = parse_target(&target);
    let cfg = read_vault_config(Path::new(&library_path));

    // MIG-108 Slice 0 — one registry read serves this library's exclusion set AND the
    // cross-library fallback below.
    let all_libs = crate::libraries::load_all_libraries(&app);
    let exclude = crate::libraries::nested_library_paths(&all_libs, &library_path);

    let res = resolve_path(&library_path, &note_path, &parsed.path, &cfg, &exclude);
    let mut matched = res.matched;
    // Cross-library fallback (Boss-found 2026-07-05): the `![[` picker offers
    // notes from EVERY registered library — mirroring wikilink resolution —
    // so a bare target that misses in the note's own library is looked up in
    // the other libraries' indexes before being declared missing.
    if matched.is_none() && !parsed.path.contains('/') && !parsed.path.contains('\\') {
        for lib in &all_libs {
            if lib.path == library_path { continue; }
            let lib_exclude = crate::libraries::nested_library_paths(&all_libs, &lib.path);
            if let Some(p) = lookup_in_library_index(&lib.path, &parsed.path, &lib_exclude) {
                matched = Some(p);
                break;
            }
        }
    }
    let Some(abs) = matched else {
        // Miss: compute "did you mean" suggestions from the vault index so the
        // user can see what files ARE present that might be a near-match.
        let similar = find_similar_in_index(&library_path, &parsed.path, &exclude);
        let vault_file_count = get_or_build_vault_index(&library_path, &exclude).len() as u64;
        let (folder_resolved, folder_listing) = list_expected_attachment_folder(&library_path, &note_path, &cfg);
        return EmbedResolution {
            kind: "missing".into(),
            url: String::new(),
            absolute_path: None,
            mime: None,
            size_bytes: 0,
            note_body: None,
            heading: parsed.heading,
            block_id: parsed.block_id,
            tried_paths: res.tried.into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
            attachment_folder: cfg.attachment_folder_path.clone(),
            similar_files: similar,
            vault_file_count,
            attachment_folder_listing: folder_listing,
            attachment_folder_resolved: folder_resolved,
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
            tried_paths: Vec::new(),
            attachment_folder: cfg.attachment_folder_path.clone(),
            similar_files: Vec::new(),
            vault_file_count: 0,
            attachment_folder_listing: Vec::new(),
            attachment_folder_resolved: String::new(),
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
            tried_paths: Vec::new(),
            attachment_folder: cfg.attachment_folder_path.clone(),
            similar_files: Vec::new(),
            vault_file_count: 0,
            attachment_folder_listing: Vec::new(),
            attachment_folder_resolved: String::new(),
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
        tried_paths: Vec::new(),
        attachment_folder: cfg.attachment_folder_path.clone(),
        similar_files: Vec::new(),
        vault_file_count: 0,
        attachment_folder_listing: Vec::new(),
        attachment_folder_resolved: String::new(),
    }
}

/// Resolve the expected attachment folder (per Obsidian rules) and list its
/// actual contents on disk. Bypasses the cached vault index — this is the
/// OS-level truth about what's in that folder right now. Used for the missing
/// diagnostic card so the user can see exactly what files are/aren't present.
///
/// Returns (resolved_absolute_path, listing). Listing is empty if the folder
/// doesn't exist or can't be read.
fn list_expected_attachment_folder(
    library_path: &str,
    note_path: &str,
    cfg: &VaultConfig,
) -> (String, Vec<String>) {
    let lib = Path::new(library_path);
    let attach = &cfg.attachment_folder_path;

    // Match the same resolution logic used in step 3 of resolve_path
    let base = if attach.is_empty() {
        lib.to_path_buf()
    } else if attach == "./" {
        Path::new(note_path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| lib.to_path_buf())
    } else if attach.starts_with("./") {
        Path::new(note_path).parent()
            .map(|p| p.join(&attach[2..]))
            .unwrap_or_else(|| lib.join(&attach[2..]))
    } else {
        lib.join(attach)
    };

    let resolved = base.to_string_lossy().into_owned();
    let Ok(entries) = fs::read_dir(&base) else {
        return (resolved, Vec::new());
    };

    let mut files: Vec<(String, bool)> = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
        files.push((name, is_dir));
    }
    // Sort: files first by name, then directories
    files.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())));
    let listing = files.into_iter()
        .take(30)
        .map(|(n, is_dir)| if is_dir { format!("[dir] {}", n) } else { n })
        .collect();
    (resolved, listing)
}

/// Find up to 8 files in the vault index whose basename resembles the target.
///
/// Matching strategy (in order):
///   1. Same file extension AND a shared 3+ char prefix (digit-normalized)
///   2. Same file extension alone (returns up to 8 arbitrary matches of that
///      type — helps the user see that e.g. "there are PNGs in the vault,
///      just not one with this name")
fn find_similar_in_index(library_path: &str, target: &str, exclude: &std::collections::HashSet<String>) -> Vec<String> {
    let index = get_or_build_vault_index(library_path, exclude);
    let target_norm = normalize_digits(&target.to_lowercase());
    let target_ext = Path::new(target).extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    let mut scored: Vec<(usize, PathBuf)> = Vec::new();
    let mut same_ext: Vec<PathBuf> = Vec::new();
    for (key, path) in index.iter() {
        let key_norm = normalize_digits(key);
        let prefix_len = target_norm.chars().zip(key_norm.chars())
            .take_while(|(a, b)| a == b).count();
        let key_ext = Path::new(key).extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        let ext_matches = target_ext.is_some() && key_ext == target_ext;
        if prefix_len >= 3 && ext_matches {
            scored.push((prefix_len, path.clone()));
        } else if ext_matches && same_ext.len() < 8 {
            same_ext.push(path.clone());
        }
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out: Vec<String> = scored.into_iter().take(8).map(|(_, p)| p.to_string_lossy().into_owned()).collect();
    // If prefix-matching gave us nothing, fall back to arbitrary files with the
    // same extension — so the user sees that e.g. there ARE images in the vault,
    // just not the one referenced.
    if out.is_empty() {
        out.extend(same_ext.into_iter().map(|p| p.to_string_lossy().into_owned()));
    }
    out
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

#[cfg(test)]
mod mig108_slice0_tests {
    use super::*;
    use std::collections::HashSet;

    fn norm_key(p: &std::path::Path) -> String {
        p.to_string_lossy().replace('\\', "/").trim_end_matches('/').to_lowercase()
    }

    /// MIG-108 Slice 0 — the vault index must stop at a nested registered library's
    /// boundary. Post-MIG-108 every library sits under the universe root, whose own
    /// universe_notes library is walked with the root as its path: without the boundary
    /// the root's index absorbed every nested library's files, and first-wins `or_insert`
    /// let a nested file SHADOW the root's own same-named file (or vice versa), silently
    /// changing which image or note an `![[embed]]` resolves to.
    #[test]
    fn vault_index_stops_at_a_nested_library_boundary() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("own.png"), b"root's own file").unwrap();
        std::fs::write(root.path().join("shared.png"), b"ROOT copy").unwrap();

        let nested = root.path().join("Nested Library");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("inner.png"), b"nested-only file").unwrap();
        std::fs::write(nested.join("shared.png"), b"NESTED copy").unwrap();

        let exclude: HashSet<String> = [norm_key(&nested)].into_iter().collect();
        let idx = build_vault_index(root.path(), &exclude);

        assert!(idx.files.contains_key("own.png"), "the root's own files are indexed");
        assert!(
            !idx.files.contains_key("inner.png"),
            "a nested REGISTERED library's files are its own, never the parent's: {:?}",
            idx.files.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            idx.files.get("shared.png").unwrap(),
            &root.path().join("shared.png"),
            "a same-named file inside the nested library must not shadow the root's own"
        );

        // And WITHOUT the exclusion the absorption is real — the guard is load-bearing.
        let unbounded = build_vault_index(root.path(), &HashSet::new());
        assert!(unbounded.files.contains_key("inner.png"), "control: no exclude → absorbed");
    }

    /// An ordinary (unregistered) subfolder is still walked — the boundary is
    /// "registered library", not "any subdirectory".
    #[test]
    fn ordinary_subfolders_are_still_indexed() {
        let root = tempfile::tempdir().unwrap();
        let sub = root.path().join("Attachments");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("pic.png"), b"x").unwrap();

        let idx = build_vault_index(root.path(), &HashSet::new());
        assert!(idx.files.contains_key("pic.png"));
    }
}
