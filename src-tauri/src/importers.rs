use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::libraries::validate_path_in_any_library;

const EXCLUDED_DIRS: &[&str] = &[
    ".obsidian", ".trash", ".git", ".svn", "node_modules", "__MACOSX",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportPreview {
    pub file_count: usize,
    pub format: String,
    pub files: Vec<ImportPreviewEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportPreviewEntry {
    pub source_name: String,
    pub target_name: String,
    pub size_bytes: u64,
}

// ─── Pick source folder/file ───────────────────────────────────────

#[tauri::command]
pub async fn import_pick_source(format: String) -> Result<String, String> {
    let dialog = rfd::AsyncFileDialog::new();
    match format.as_str() {
        "folder" | "markdown" | "bear" | "notion" | "obsidian" => {
            let folder = dialog
                .set_title("Select folder to import")
                .pick_folder()
                .await;
            match folder {
                Some(f) => Ok(f.path().to_string_lossy().to_string()),
                None => Err("No folder selected".to_string()),
            }
        }
        "enex" => {
            let file = dialog
                .set_title("Select Evernote .enex file")
                .add_filter("Evernote Export", &["enex"])
                .pick_file()
                .await;
            match file {
                Some(f) => Ok(f.path().to_string_lossy().to_string()),
                None => Err("No file selected".to_string()),
            }
        }
        "html" => {
            let file = dialog
                .set_title("Select HTML file(s)")
                .add_filter("HTML", &["html", "htm"])
                .pick_files()
                .await;
            match file {
                Some(files) => {
                    let paths: Vec<String> = files
                        .iter()
                        .map(|f| f.path().to_string_lossy().to_string())
                        .collect();
                    Ok(paths.join("|"))
                }
                None => Err("No files selected".to_string()),
            }
        }
        "csv" => {
            let file = dialog
                .set_title("Select CSV file")
                .add_filter("CSV", &["csv", "tsv"])
                .pick_file()
                .await;
            match file {
                Some(f) => Ok(f.path().to_string_lossy().to_string()),
                None => Err("No file selected".to_string()),
            }
        }
        "txt" => {
            let file = dialog
                .set_title("Select text files")
                .add_filter("Text", &["txt", "text"])
                .pick_files()
                .await;
            match file {
                Some(files) => {
                    let paths: Vec<String> = files
                        .iter()
                        .map(|f| f.path().to_string_lossy().to_string())
                        .collect();
                    Ok(paths.join("|"))
                }
                None => Err("No files selected".to_string()),
            }
        }
        _ => Err(format!("Unknown format: {}", format)),
    }
}

// ─── Preview import ───────────────────────────────────────

#[tauri::command]
pub async fn import_preview(
    source: String,
    format: String,
) -> Result<ImportPreview, String> {
    match format.as_str() {
        "markdown" | "folder" | "bear" | "obsidian" => preview_folder_all(&source),
        "notion" => preview_notion(&source),
        "enex" => preview_enex(&source),
        "html" => preview_multi_files(&source, "html"),
        "csv" => preview_csv(&source),
        "txt" => preview_multi_files(&source, "txt"),
        _ => Err(format!("Unknown format: {}", format)),
    }
}

fn preview_folder_all(source: &str) -> Result<ImportPreview, String> {
    let path = Path::new(source);
    if !path.is_dir() {
        return Err("Source is not a directory".to_string());
    }
    let mut entries = Vec::new();
    collect_all_files_recursive(path, &mut entries)?;
    Ok(ImportPreview {
        file_count: entries.len(),
        format: "markdown".to_string(),
        files: entries,
    })
}

fn preview_notion(source: &str) -> Result<ImportPreview, String> {
    let path = Path::new(source);
    if !path.is_dir() {
        return Err("Source is not a directory".to_string());
    }
    let mut entries = Vec::new();
    collect_all_files_recursive(path, &mut entries)?;
    Ok(ImportPreview {
        file_count: entries.len(),
        format: "notion".to_string(),
        files: entries,
    })
}

fn preview_enex(source: &str) -> Result<ImportPreview, String> {
    let content = fs::read_to_string(source).map_err(|e| e.to_string())?;
    let count = content.matches("<note>").count();
    let size = fs::metadata(source).map(|m| m.len()).unwrap_or(0);
    let entries: Vec<ImportPreviewEntry> = (0..count)
        .map(|i| {
            // Extract title
            let title = extract_between(&content, "<title>", "</title>", i)
                .unwrap_or_else(|| format!("Note {}", i + 1));
            ImportPreviewEntry {
                source_name: title.clone(),
                target_name: sanitize_filename(&title) + ".md",
                size_bytes: size / count.max(1) as u64,
            }
        })
        .collect();
    Ok(ImportPreview {
        file_count: count,
        format: "enex".to_string(),
        files: entries,
    })
}

fn preview_multi_files(source: &str, fmt: &str) -> Result<ImportPreview, String> {
    let paths: Vec<&str> = source.split('|').collect();
    let mut entries = Vec::new();
    for p in &paths {
        let path = Path::new(p);
        if path.is_file() {
            let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            entries.push(ImportPreviewEntry {
                source_name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                target_name: sanitize_filename(&name) + ".md",
                size_bytes: size,
            });
        }
    }
    Ok(ImportPreview {
        file_count: entries.len(),
        format: fmt.to_string(),
        files: entries,
    })
}

fn preview_csv(source: &str) -> Result<ImportPreview, String> {
    let size = fs::metadata(source).map(|m| m.len()).unwrap_or(0);
    let content = fs::read_to_string(source).map_err(|e| e.to_string())?;
    let line_count = content.lines().count().saturating_sub(1); // minus header
    let name = Path::new(source)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    Ok(ImportPreview {
        file_count: line_count,
        format: "csv".to_string(),
        files: vec![ImportPreviewEntry {
            source_name: format!("{} ({} rows)", name, line_count),
            target_name: format!("{} notes from CSV", line_count),
            size_bytes: size,
        }],
    })
}

// ─── Execute import ───────────────────────────────────────

#[tauri::command]
pub async fn import_execute(
    app: tauri::AppHandle,
    source: String,
    format: String,
    target_library: String,
    subfolder: String,
) -> Result<ImportResult, String> {
    // Validate target library path
    validate_path_in_any_library(&app, &target_library)?;

    let dest = if subfolder.is_empty() {
        PathBuf::from(&target_library)
    } else {
        PathBuf::from(&target_library).join(&subfolder)
    };

    // Create destination folder
    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create destination: {}", e))?;

    match format.as_str() {
        "markdown" | "folder" | "bear" | "obsidian" => import_markdown_folder(&source, &dest),
        "notion" => import_notion_folder(&source, &dest),
        "enex" => import_enex(&source, &dest),
        "html" => import_html_files(&source, &dest),
        "csv" => import_csv(&source, &dest),
        "txt" => import_text_files(&source, &dest),
        _ => Err(format!("Unknown format: {}", format)),
    }
}

fn import_markdown_folder(source: &str, dest: &Path) -> Result<ImportResult, String> {
    let src = Path::new(source);
    let mut result = ImportResult { imported: 0, skipped: 0, errors: vec![], files: vec![] };
    copy_full_tree(src, dest, src, &mut result)?;
    Ok(result)
}

fn copy_full_tree(
    current: &Path,
    dest_base: &Path,
    src_root: &Path,
    result: &mut ImportResult,
) -> Result<(), String> {
    let entries = fs::read_dir(current).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(src_root).unwrap_or(&path);
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        if path.is_dir() {
            if name.starts_with('.') || EXCLUDED_DIRS.iter().any(|d| name.eq_ignore_ascii_case(d)) {
                continue;
            }
            let sub_dest = dest_base.join(rel);
            let _ = fs::create_dir_all(&sub_dest);
            copy_full_tree(&path, dest_base, src_root, result)?;
        } else {
            let target = dest_base.join(rel);
            if let Some(parent) = target.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if target.exists() {
                result.skipped += 1;
            } else {
                match fs::copy(&path, &target) {
                    Ok(_) => {
                        result.imported += 1;
                        result.files.push(target.to_string_lossy().to_string());
                    }
                    Err(e) => result.errors.push(format!("{}: {}", path.display(), e)),
                }
            }
        }
    }
    Ok(())
}

fn import_notion_folder(source: &str, dest: &Path) -> Result<ImportResult, String> {
    let src = Path::new(source);
    let mut result = ImportResult { imported: 0, skipped: 0, errors: vec![], files: vec![] };

    fn walk_notion(dir: &Path, dest: &Path, result: &mut ImportResult) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_notion(&path, dest, result);
            } else if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if ext_lower == "md" || ext_lower == "markdown" {
                    let raw_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    // Notion appends a hex ID to filenames like "Page Title abc123def"
                    let clean_name = clean_notion_name(&raw_name);
                    let target = dest.join(format!("{}.md", sanitize_filename(&clean_name)));
                    if target.exists() {
                        result.skipped += 1;
                    } else {
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                let cleaned = clean_notion_content(&content);
                                match fs::write(&target, cleaned) {
                                    Ok(_) => {
                                        result.imported += 1;
                                        result.files.push(target.to_string_lossy().to_string());
                                    }
                                    Err(e) => result.errors.push(format!("{}: {}", path.display(), e)),
                                }
                            }
                            Err(e) => result.errors.push(format!("{}: {}", path.display(), e)),
                        }
                    }
                } else if ext_lower != "csv" {
                    // Copy attachments (images, PDFs, etc.)
                    let target = dest.join(path.file_name().unwrap_or_default());
                    if !target.exists() {
                        match fs::copy(&path, &target) {
                            Ok(_) => { result.imported += 1; }
                            Err(e) => result.errors.push(format!("{}: {}", path.display(), e)),
                        }
                    }
                }
            }
        }
    }

    walk_notion(src, dest, &mut result);
    Ok(result)
}

fn import_enex(source: &str, dest: &Path) -> Result<ImportResult, String> {
    let content = fs::read_to_string(source).map_err(|e| e.to_string())?;
    let mut result = ImportResult { imported: 0, skipped: 0, errors: vec![], files: vec![] };

    let mut search_from = 0;
    while let Some(start) = content[search_from..].find("<note>") {
        let abs_start = search_from + start;
        let end = content[abs_start..]
            .find("</note>")
            .map(|e| abs_start + e + 7)
            .unwrap_or(content.len());
        let note_xml = &content[abs_start..end];

        let title = extract_xml_tag(note_xml, "title").unwrap_or_else(|| "Untitled".to_string());
        let enml_content = extract_xml_tag(note_xml, "content").unwrap_or_default();
        let created = extract_xml_tag(note_xml, "created").unwrap_or_default();
        let tags: Vec<String> = extract_all_xml_tags(note_xml, "tag");

        // Convert ENML to markdown
        let md = enml_to_markdown(&enml_content);

        // Build frontmatter
        let mut frontmatter = String::from("---\n");
        if !created.is_empty() {
            if let Some(date) = parse_evernote_date(&created) {
                frontmatter.push_str(&format!("created: {}\n", date));
            }
        }
        if !tags.is_empty() {
            frontmatter.push_str("tags:\n");
            for tag in &tags {
                frontmatter.push_str(&format!("  - {}\n", tag));
            }
        }
        frontmatter.push_str("---\n\n");

        let filename = sanitize_filename(&title) + ".md";
        let target = dest.join(&filename);
        if target.exists() {
            result.skipped += 1;
        } else {
            let full_content = format!("{}{}", frontmatter, md);
            match fs::write(&target, full_content) {
                Ok(_) => {
                    result.imported += 1;
                    result.files.push(target.to_string_lossy().to_string());
                }
                Err(e) => result.errors.push(format!("{}: {}", title, e)),
            }
        }

        search_from = end;
    }

    Ok(result)
}

fn import_html_files(source: &str, dest: &Path) -> Result<ImportResult, String> {
    let paths: Vec<&str> = source.split('|').collect();
    let mut result = ImportResult { imported: 0, skipped: 0, errors: vec![], files: vec![] };

    for p in paths {
        let path = Path::new(p);
        if !path.is_file() {
            continue;
        }
        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let target = dest.join(format!("{}.md", sanitize_filename(&name)));

        if target.exists() {
            result.skipped += 1;
            continue;
        }

        match fs::read_to_string(path) {
            Ok(html) => {
                let md = html_to_markdown(&html);
                match fs::write(&target, md) {
                    Ok(_) => {
                        result.imported += 1;
                        result.files.push(target.to_string_lossy().to_string());
                    }
                    Err(e) => result.errors.push(format!("{}: {}", name, e)),
                }
            }
            Err(e) => result.errors.push(format!("{}: {}", name, e)),
        }
    }

    Ok(result)
}

fn import_csv(source: &str, dest: &Path) -> Result<ImportResult, String> {
    let content = fs::read_to_string(source).map_err(|e| e.to_string())?;
    let mut result = ImportResult { imported: 0, skipped: 0, errors: vec![], files: vec![] };

    let mut lines = content.lines();
    let header_line = lines.next().ok_or("CSV is empty")?;
    let headers: Vec<&str> = parse_csv_row(header_line);

    if headers.is_empty() {
        return Err("CSV has no columns".to_string());
    }

    for (idx, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = parse_csv_row(line);
        let title = cols.first().unwrap_or(&"").to_string();
        let display_title = if title.is_empty() {
            format!("Row {}", idx + 1)
        } else {
            title.clone()
        };

        // Build frontmatter from columns
        let mut frontmatter = String::from("---\n");
        for (i, header) in headers.iter().enumerate() {
            if i == 0 {
                continue; // skip title column
            }
            let val = cols.get(i).unwrap_or(&"");
            if !val.is_empty() {
                frontmatter.push_str(&format!("{}: {}\n", header.trim(), val.trim()));
            }
        }
        frontmatter.push_str("---\n\n");

        let filename = sanitize_filename(&display_title) + ".md";
        let target = dest.join(&filename);
        if target.exists() {
            result.skipped += 1;
        } else {
            let body = format!("{}# {}\n", frontmatter, display_title);
            match fs::write(&target, body) {
                Ok(_) => {
                    result.imported += 1;
                    result.files.push(target.to_string_lossy().to_string());
                }
                Err(e) => result.errors.push(format!("{}: {}", display_title, e)),
            }
        }
    }

    Ok(result)
}

fn import_text_files(source: &str, dest: &Path) -> Result<ImportResult, String> {
    let paths: Vec<&str> = source.split('|').collect();
    let mut result = ImportResult { imported: 0, skipped: 0, errors: vec![], files: vec![] };

    for p in paths {
        let path = Path::new(p);
        if !path.is_file() {
            continue;
        }
        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let target = dest.join(format!("{}.md", sanitize_filename(&name)));

        if target.exists() {
            result.skipped += 1;
            continue;
        }

        match fs::read_to_string(path) {
            Ok(text) => {
                let md = format!("# {}\n\n{}", name, text);
                match fs::write(&target, md) {
                    Ok(_) => {
                        result.imported += 1;
                        result.files.push(target.to_string_lossy().to_string());
                    }
                    Err(e) => result.errors.push(format!("{}: {}", name, e)),
                }
            }
            Err(e) => result.errors.push(format!("{}: {}", name, e)),
        }
    }

    Ok(result)
}

// ─── Helpers ───────────────────────────────────────────────

fn collect_all_files_recursive(
    dir: &Path,
    entries: &mut Vec<ImportPreviewEntry>,
) -> Result<(), String> {
    let read = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in read.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if path.is_dir() {
            if name.starts_with('.') || EXCLUDED_DIRS.iter().any(|d| name.eq_ignore_ascii_case(d)) {
                continue;
            }
            collect_all_files_recursive(&path, entries)?;
        } else {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let file_name = name.to_string();
            entries.push(ImportPreviewEntry {
                source_name: file_name.clone(),
                target_name: file_name,
                size_bytes: size,
            });
        }
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|' {
                '_'
            } else {
                c
            }
        })
        .collect();
    s = s.trim().to_string();
    if s.is_empty() {
        s = "Untitled".to_string();
    }
    // Limit length
    if s.len() > 200 {
        s.truncate(200);
    }
    s
}

fn extract_between(text: &str, open: &str, close: &str, occurrence: usize) -> Option<String> {
    let mut count = 0;
    let mut search_from = 0;
    loop {
        let start = text[search_from..].find(open)?;
        let abs_start = search_from + start + open.len();
        let end = text[abs_start..].find(close)?;
        if count == occurrence {
            return Some(text[abs_start..abs_start + end].to_string());
        }
        count += 1;
        search_from = abs_start + end + close.len();
    }
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    // Also handle CDATA
    if let Some(start) = xml.find(&open) {
        let content_start = start + open.len();
        if let Some(end) = xml[content_start..].find(&close) {
            let raw = &xml[content_start..content_start + end];
            // Strip CDATA
            let cleaned = raw
                .trim()
                .strip_prefix("<![CDATA[")
                .and_then(|s| s.strip_suffix("]]>"))
                .unwrap_or(raw);
            return Some(cleaned.to_string());
        }
    }
    None
}

fn extract_all_xml_tags(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let mut results = Vec::new();
    let mut search_from = 0;
    while let Some(start) = xml[search_from..].find(&open) {
        let abs_start = search_from + start + open.len();
        if let Some(end) = xml[abs_start..].find(&close) {
            results.push(xml[abs_start..abs_start + end].trim().to_string());
            search_from = abs_start + end + close.len();
        } else {
            break;
        }
    }
    results
}

fn parse_evernote_date(date: &str) -> Option<String> {
    // Evernote format: 20230415T123456Z
    if date.len() >= 8 {
        let year = &date[0..4];
        let month = &date[4..6];
        let day = &date[6..8];
        Some(format!("{}-{}-{}", year, month, day))
    } else {
        None
    }
}

fn enml_to_markdown(enml: &str) -> String {
    let mut md = enml.to_string();

    // Remove XML declaration and DTD
    if let Some(pos) = md.find("?>") {
        md = md[pos + 2..].to_string();
    }
    if let Some(pos) = md.find(">") {
        if md[..pos].contains("DOCTYPE") || md[..pos].contains("en-note") {
            md = md[pos + 1..].to_string();
        }
    }

    // Strip en-note wrapper
    md = md.replace("<en-note>", "").replace("</en-note>", "");
    md = md.replace("<en-note/>", "");

    // Convert common HTML tags
    md = convert_html_tags(&md);

    // Clean up
    md = md.trim().to_string();
    md
}

fn html_to_markdown(html: &str) -> String {
    let mut md = html.to_string();

    // Remove <head> section
    if let (Some(start), Some(end)) = (md.find("<head"), md.find("</head>")) {
        md = format!("{}{}", &md[..start], &md[end + 7..]);
    }

    // Extract body content if present
    if let Some(body_start) = md.find("<body") {
        if let Some(gt) = md[body_start..].find('>') {
            let content_start = body_start + gt + 1;
            let content_end = md.find("</body>").unwrap_or(md.len());
            md = md[content_start..content_end].to_string();
        }
    }

    md = convert_html_tags(&md);
    md = md.trim().to_string();
    md
}

fn convert_html_tags(html: &str) -> String {
    let mut md = html.to_string();

    // Headings
    let heading_re = regex::Regex::new(r"<h([1-6])[^>]*>(.*?)</h\1>").unwrap();
    md = heading_re
        .replace_all(&md, |caps: &regex::Captures| {
            let level: usize = caps[1].parse().unwrap_or(1);
            let text = strip_tags(&caps[2]);
            format!("{} {}", "#".repeat(level), text.trim())
        })
        .to_string();

    // Bold
    md = regex::Regex::new(r"<(b|strong)[^>]*>(.*?)</\1>")
        .unwrap()
        .replace_all(&md, "**$2**")
        .to_string();

    // Italic
    md = regex::Regex::new(r"<(i|em)[^>]*>(.*?)</\1>")
        .unwrap()
        .replace_all(&md, "*$2*")
        .to_string();

    // Links
    md = regex::Regex::new(r#"<a[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#)
        .unwrap()
        .replace_all(&md, "[$2]($1)")
        .to_string();

    // Images
    md = regex::Regex::new(r#"<img[^>]*src="([^"]*)"[^>]*/?\s*>"#)
        .unwrap()
        .replace_all(&md, "![]($1)")
        .to_string();

    // Lists
    md = regex::Regex::new(r"<li[^>]*>(.*?)</li>")
        .unwrap()
        .replace_all(&md, "- $1")
        .to_string();
    md = regex::Regex::new(r"</?[ou]l[^>]*>")
        .unwrap()
        .replace_all(&md, "")
        .to_string();

    // Code blocks
    md = regex::Regex::new(r"<pre[^>]*><code[^>]*>([\s\S]*?)</code></pre>")
        .unwrap()
        .replace_all(&md, "```\n$1\n```")
        .to_string();

    // Inline code
    md = regex::Regex::new(r"<code[^>]*>(.*?)</code>")
        .unwrap()
        .replace_all(&md, "`$1`")
        .to_string();

    // Blockquotes
    md = regex::Regex::new(r"<blockquote[^>]*>(.*?)</blockquote>")
        .unwrap()
        .replace_all(&md, "> $1")
        .to_string();

    // HR
    md = regex::Regex::new(r"<hr[^>]*/?\s*>")
        .unwrap()
        .replace_all(&md, "\n---\n")
        .to_string();

    // Paragraphs and line breaks
    md = regex::Regex::new(r"<br[^>]*/?\s*>")
        .unwrap()
        .replace_all(&md, "\n")
        .to_string();
    md = regex::Regex::new(r"<p[^>]*>(.*?)</p>")
        .unwrap()
        .replace_all(&md, "$1\n\n")
        .to_string();
    md = regex::Regex::new(r"<div[^>]*>(.*?)</div>")
        .unwrap()
        .replace_all(&md, "$1\n")
        .to_string();

    // Strip remaining HTML tags
    md = strip_tags(&md);

    // Decode common entities
    md = md
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ");

    // Clean up multiple blank lines
    while md.contains("\n\n\n") {
        md = md.replace("\n\n\n", "\n\n");
    }

    md
}

fn strip_tags(html: &str) -> String {
    regex::Regex::new(r"<[^>]+>")
        .unwrap()
        .replace_all(html, "")
        .to_string()
}

fn clean_notion_name(name: &str) -> String {
    // Notion appends a 32-char hex ID, e.g. "My Page a1b2c3d4e5f6..."
    let re = regex::Regex::new(r"\s+[a-f0-9]{32}$").unwrap();
    re.replace(name, "").trim().to_string()
}

fn clean_notion_content(content: &str) -> String {
    let mut out = content.to_string();
    // Remove Notion-specific properties block at start
    // Usually starts with metadata lines before first heading
    // Keep content as-is for markdown files, just clean up links
    let notion_link_re = regex::Regex::new(r"\[([^\]]+)\]\([^)]*notion\.so[^)]*\)").unwrap();
    out = notion_link_re.replace_all(&out, "[[$1]]").to_string();
    out
}

fn parse_csv_row(line: &str) -> Vec<&str> {
    // Simple CSV parsing (doesn't handle quoted fields with commas)
    let sep = if line.contains('\t') { '\t' } else { ',' };
    line.split(sep).collect()
}

// ─── Canonical Import Pipeline ───────────────────────────────────────

use crate::canonical::{
    file_creation_time, generate_canonical, inject_frontmatter, write_sidecar, FrontmatterFields,
    SidecarMetadata,
};
use crate::file_kinds::{classify_file, KindRegistry};

/// Import files with canonical filenames + classification + frontmatter enrichment.
/// This is the full pipeline: scan → classify → generate canonical names → enrich → write.
#[tauri::command]
pub async fn import_with_canonical(
    app: tauri::AppHandle,
    source: String,
    format: String,
    target_library: String,
    subfolder: String,
) -> Result<ImportResult, String> {
    crate::libraries::validate_path_in_any_library(&app, &target_library)?;

    let dest = if subfolder.is_empty() {
        PathBuf::from(&target_library)
    } else {
        PathBuf::from(&target_library).join(&subfolder)
    };
    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create destination: {}", e))?;

    // Get kind registry config path
    let config_path = crate::universe::active_constellation_dir(&app)
        .map(|d| d.join("file_kinds.json"))
        .ok();
    let mut registry = KindRegistry::new(config_path.as_deref());

    let mut result = ImportResult {
        imported: 0,
        skipped: 0,
        errors: vec![],
        files: vec![],
    };

    // Phase 1: Collect source files based on format
    let source_files = match format.as_str() {
        "markdown" | "folder" | "obsidian" => collect_source_files_recursive(Path::new(&source)),
        "notion" => collect_source_files_recursive(Path::new(&source)),
        _ => {
            // For other formats (enex, html, csv, txt), use the legacy pipeline
            // They already produce .md files which can be canonicalized afterwards
            return import_execute_legacy(&app, source, format, target_library, subfolder).await;
        }
    };

    // Phase 2-5: Classify, generate canonical names, enrich, write
    for src_file in &source_files {
        let src_path = Path::new(src_file);
        if !src_path.is_file() {
            continue;
        }

        let ext = src_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Classify the file
        let kind = classify_file(src_path, &mut registry);

        // Get creation timestamp
        let created = file_creation_time(src_path);

        // Generate canonical filename
        let file_ext = if ext.is_empty() { "bin".to_string() } else { ext.clone() };
        let canonical = generate_canonical(&kind, &created, &file_ext, Some(&dest));
        let dest_path = dest.join(&canonical.full);

        // Get human-readable title from original filename
        let original_name = src_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let display_name = src_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Clean up Notion UUID suffixes if needed
        let clean_title = if format == "notion" {
            clean_notion_name(&display_name)
        } else {
            display_name.clone()
        };

        if ext == "md" || ext == "markdown" {
            // Read content, enrich with frontmatter, write with canonical name
            match fs::read_to_string(src_path) {
                Ok(mut content) => {
                    // Clean Notion content if needed
                    if format == "notion" {
                        content = clean_notion_content(&content);
                    }

                    let fields = FrontmatterFields {
                        title: clean_title.clone(),
                        cid: canonical.stem.clone(),
                        kind: kind.to_lowercase(),
                        created: created.to_rfc3339(),
                        aliases: vec![clean_title],
                        original_filename: Some(original_name),
                    };
                    let enriched = inject_frontmatter(&content, &fields);

                    match fs::write(&dest_path, enriched) {
                        Ok(_) => {
                            result.imported += 1;
                            result.files.push(dest_path.to_string_lossy().to_string());
                        }
                        Err(e) => result.errors.push(format!("{}: {}", src_path.display(), e)),
                    }
                }
                Err(e) => result.errors.push(format!("{}: {}", src_path.display(), e)),
            }
        } else {
            // Non-markdown: copy file + create sidecar
            match fs::copy(src_path, &dest_path) {
                Ok(_) => {
                    result.imported += 1;
                    result.files.push(dest_path.to_string_lossy().to_string());

                    // Create sidecar metadata
                    let sidecar = SidecarMetadata {
                        title: clean_title.clone(),
                        cid: canonical.stem.clone(),
                        kind: kind.to_lowercase(),
                        created: created.to_rfc3339(),
                        original_filename: original_name,
                        aliases: vec![clean_title],
                        referenced_by: Vec::new(),
                    };
                    if let Err(e) = write_sidecar(&dest_path, &sidecar) {
                        result.errors.push(e);
                    }
                }
                Err(e) => result.errors.push(format!("{}: {}", src_path.display(), e)),
            }
        }
    }

    // Write canonical marker if we imported anything
    if result.imported > 0 {
        let marker_dir = PathBuf::from(&target_library).join(".constellation");
        let _ = fs::create_dir_all(&marker_dir);
        let _ = fs::write(
            marker_dir.join("canonical"),
            format!(
                "Canonicalized on {}\nFiles imported: {}\n",
                chrono::Utc::now().to_rfc3339(),
                result.imported
            ),
        );
    }

    Ok(result)
}

/// Legacy import (delegates to the original import_execute).
async fn import_execute_legacy(
    _app: &tauri::AppHandle,
    source: String,
    format: String,
    target_library: String,
    subfolder: String,
) -> Result<ImportResult, String> {
    let dest = if subfolder.is_empty() {
        PathBuf::from(&target_library)
    } else {
        PathBuf::from(&target_library).join(&subfolder)
    };
    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create destination: {}", e))?;

    match format.as_str() {
        "enex" => import_enex(&source, &dest),
        "html" => import_html_files(&source, &dest),
        "csv" => import_csv(&source, &dest),
        "txt" => import_text_files(&source, &dest),
        _ => Err(format!("Unknown format for legacy import: {}", format)),
    }
}

/// Collect all file paths recursively from a source directory.
fn collect_source_files_recursive(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if path.is_dir() {
            if name.starts_with('.') || EXCLUDED_DIRS.iter().any(|d| name.eq_ignore_ascii_case(d)) {
                continue;
            }
            files.extend(collect_source_files_recursive(&path));
        } else {
            files.push(path.to_string_lossy().to_string());
        }
    }
    files
}
