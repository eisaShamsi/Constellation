use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::libraries::validate_path_in_any_library;
use crate::file_kinds::EXCLUDED_DIRS;

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

/// PJ-407 — a note must not be COPIED IN under a name the app can never read.
///
/// `copy_full_tree`'s directory arm already refuses dot-directories; its FILE arm did not, so a
/// vault holding `.NET.md` landed `.NET.md` in the user's library and the note was invisible to the
/// whole app the moment it arrived — every walker skips dot-names, so it could neither be indexed
/// NOR reported as missing. That is the same door `sanitize_filename` closes for the other
/// importers, left open for the four formats that route through `copy_full_tree`
/// (`markdown | folder | bear | obsidian`, dispatched at the `import_from_source` match).
/// `obsidian` is the default selection in the first-run Universe Setup wizard, so this is the
/// likeliest import a new user ever performs.
///
/// Deliberately NARROW, and deliberately NOT `sanitize_filename`:
/// * only the LEAF is rewritten, so the source's folder structure is reproduced unchanged;
/// * only when the leaf is a `.md` file, so `.gitignore`, `.DS_Store` and any other dotfile the
///   user chose to carry across are copied through untouched — they are not notes, and renaming
///   them would corrupt a vault;
/// * `sanitize_filename` would additionally truncate at 200 bytes, and a collision produced that
///   way is reported by `copy_full_tree` as `skipped` rather than as an error — a silent loss.
///
/// Measured with a `rustc` probe rather than assumed (2026-08-27): `Path::extension()` is `None`
/// for `.md`, `.gitignore` and `.DS_Store` (a name whose only dot is the leading one), and
/// `Some("md")` for `..md` / `...md`. The second check rejects that second group: trimming them
/// would produce a bare `md` with no extension — still unindexable, and less honest than the junk
/// name it arrived under.
fn unhide_md_leaf(rel: &Path) -> std::borrow::Cow<'_, Path> {
    let leaf = match rel.file_name().and_then(|n| n.to_str()) {
        Some(l) if l.starts_with('.') => l,
        _ => return std::borrow::Cow::Borrowed(rel),
    };
    let is_md = |n: &str| {
        Path::new(n)
            .extension()
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
    };
    if !is_md(leaf) {
        return std::borrow::Cow::Borrowed(rel);
    }
    let fixed = leaf.trim_start_matches('.');
    if !is_md(fixed) {
        return std::borrow::Cow::Borrowed(rel);
    }
    match rel.parent() {
        Some(pp) if !pp.as_os_str().is_empty() => std::borrow::Cow::Owned(pp.join(fixed)),
        _ => std::borrow::Cow::Owned(PathBuf::from(fixed)),
    }
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
            let rel = unhide_md_leaf(rel);
            let target = dest_base.join(rel.as_ref());
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
                // MIG-112 §1 — the same guard its three siblings in this file already apply
                // (`copy_full_tree`, `collect_all_files_recursive`, `collect_source_files_recursive`).
                // A Notion export unzipped over an existing folder can carry `.git/`,
                // `__MACOSX/` or `node_modules/`, and without this those are imported as notes.
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || EXCLUDED_DIRS.iter().any(|d| name.eq_ignore_ascii_case(d)) {
                    continue;
                }
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
                                match crate::write_gate::gate_write(&target, &cleaned, None, "import") {
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
            match crate::write_gate::gate_write(&target, &full_content, None, "import") {
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
                match crate::write_gate::gate_write(&target, &md, None, "import") {
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
            match crate::write_gate::gate_write(&target, &body, None, "import") {
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
                match crate::write_gate::gate_write(&target, &md, None, "import") {
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
    // PJ-407 — TRIM LEADING/TRAILING DOTS, exactly as `libraries::note_display_filename` does
    // for a note the user creates.
    //
    // A dot-prefixed name is HIDDEN — POSIX convention, and Obsidian's explicit design decision
    // ("Files and folders beginning with a `.` are intended to be hidden", and asked what the fix
    // would be: "most likely preventing users from creating .dot files"; forum.obsidian.md/t/
    // markdown-files-and-folders-with-leading-dots-are-not-visible/1622, WhiteNoise, 2020-06-13
    // and 2020-07-09 — the thread's opening report is a file named `.NET Framework.md`). This
    // door copied such names through verbatim, and the note landed on disk invisible to the
    // ENTIRE app: every walker skips dot-names (`search.rs:9041` in the indexer,
    // `reconcile.rs` in the orphan check), so it could neither be indexed NOR detected as
    // missing — invisible to the very pass built to find invisible notes.
    //
    // DO NOT READ THIS AS "the dot rule is now closed." It is not, and an earlier version of
    // this comment said it was — a false structural claim, caught by the review panel on the day
    // it was written. Closed here and in `unhide_md_leaf` (the folder/Obsidian import) and
    // `universe::sanitize_template_stem` ("Save as template"). STILL OPEN, each filed with its
    // own PJ because each needs a refuse-or-strip decision and fifteen locale strings: renaming
    // a note or a folder (`libraries::rename_item`), New Folder (`libraries::sanitize_name`),
    // New Library (`create_new_library_at`), and the daily-note / quick-capture folder fields.
    //
    // Measured on the Boss's machine before this fix: `.NET.md` (23,227 B) and
    // `.NET Framework.md` (35,578 B), both real notes with frontmatter and tags, in
    // `Computer Science\Algorithms & Data Structures`. Wikipedia-derived, so exactly the shape an
    // import produces. Renaming at the door is what Obsidian would do; what Constellation adds is
    // that it does not do it in silence (see the drift count).
    s = s.trim().trim_matches('.').trim().to_string();
    if s.is_empty() {
        s = "Untitled".to_string();
    }
    // Limit length.
    //
    // PJ-409 — `String::truncate` PANICS when the byte index is not a UTF-8 char boundary, and an
    // import is exactly where non-ASCII titles arrive; a panic here aborts the command mid-loop,
    // after some notes have already been written. Fixed in-pass because it is inside the function
    // this change edits and a panic on the Boss's Arabic notes is not something to file for later.
    if s.len() > 200 {
        let mut cut = 200;
        while !s.is_char_boundary(cut) {
            cut -= 1;
        }
        s.truncate(cut);
        s = s.trim_end().to_string();
    }
    s
}

#[cfg(test)]
mod tests_pj407_import_filename {
    //! PJ-407 — an imported note must not land on disk with a name the app can never see.
    //!
    //! Researched before fixing (WA#5). Obsidian's position, from a thread now in its Bug
    //! graveyard: *"This is not a bug but a design decision. Files and folders beginning with a
    //! `.` are intended to be hidden"*, and the fix they would choose is *"most likely preventing
    //! users from creating .dot files."* So the convention stays — it is also what keeps
    //! `.trash`, `.obsidian`, `.git` and `.constellation` out of the index — and prevention moves
    //! to the door. Constellation already prevented it for a note the USER creates
    //! (`libraries::note_display_filename` trims dots); this door did not. It was not the only
    //! one — see the list in `sanitize_filename`'s comment. Saying otherwise is what the first
    //! version of this pass did, and the panel refuted it the same day.
    use super::sanitize_filename;

    #[test]
    fn a_leading_dot_is_stripped_so_the_note_is_not_born_invisible() {
        // The Boss's actual files, measured on his machine before the fix: 23,227 B and 35,578 B
        // of real content, in `Computer Science\Algorithms & Data Structures`, indexed nowhere.
        assert_eq!(sanitize_filename(".NET"), "NET");
        assert_eq!(sanitize_filename(".NET Framework"), "NET Framework");
        assert_eq!(sanitize_filename("..hidden"), "hidden");
    }

    #[test]
    fn a_name_that_is_only_dots_falls_back_rather_than_becoming_empty() {
        assert_eq!(sanitize_filename("."), "Untitled");
        assert_eq!(sanitize_filename("..."), "Untitled");
    }

    #[test]
    fn an_ordinary_name_is_untouched_and_inner_dots_survive() {
        // The fix must not mangle the common case: version numbers, file-ish titles, abbreviations.
        assert_eq!(sanitize_filename("Node.js"), "Node.js");
        assert_eq!(sanitize_filename("v1.2.3 release notes"), "v1.2.3 release notes");
        assert_eq!(sanitize_filename("Ordinary Note"), "Ordinary Note");
        assert_eq!(sanitize_filename("الكيماويات السامة"), "الكيماويات السامة");
    }

    /// The two doors must agree **on the dot rule**. They do NOT agree in general, and saying
    /// otherwise would be the over-claim this test exists to prevent: `note_display_filename`
    /// replaces reserved characters with a space and collapses runs, guards Windows reserved
    /// names (`CON` -> `CON_`), and truncates at 240 bytes; `sanitize_filename` substitutes `_`,
    /// has no reserved-name guard, and truncates at 200. Only the leading-dot behaviour is
    /// pinned here — which is the one that made a note invisible.
    #[test]
    fn the_two_doors_agree_on_the_dot_rule() {
        for t in [".NET", ".NET Framework", "..hidden", "Node.js", "Ordinary Note"] {
            assert_eq!(
                sanitize_filename(t),
                crate::libraries::note_display_filename(t),
                "import and creation must agree on the DOT RULE for {t:?}"
            );
        }
    }

    /// PJ-407 second door — the folder / Obsidian / Bear / Markdown import copies a tree
    /// verbatim; it never sees `sanitize_filename`. Found by the review panel AFTER the first fix
    /// shipped a comment claiming "this one line is the whole defect." It was not — this door,
    /// and the template namer, were both open, and four more remain open behind a filed decision.
    #[test]
    fn a_copied_tree_does_not_carry_a_hidden_note_in_with_it() {
        use super::unhide_md_leaf;
        use std::path::{Path, PathBuf};
        let f = |s: &str| unhide_md_leaf(Path::new(s)).into_owned();

        // The Boss's own two files, as they would arrive inside an imported vault.
        assert_eq!(f(".NET.md"), PathBuf::from("NET.md"));
        assert_eq!(f(".NET Framework.md"), PathBuf::from("NET Framework.md"));
        // Nested: the folder structure must be reproduced exactly, only the leaf changes.
        assert_eq!(
            f("Computer Science/Algorithms/.NET.md"),
            PathBuf::from("Computer Science/Algorithms/NET.md")
        );

        // NOT notes. A vault legitimately carries these and renaming them would corrupt it.
        for keep in [".gitignore", ".DS_Store", ".obsidian/app.json", "notes/.gitignore"] {
            assert_eq!(f(keep), PathBuf::from(keep), "{keep} must be copied through untouched");
        }
        // Junk whose only dot-stripped form would lose the extension — leave it as it came.
        for keep in [".md", "..md", "...md"] {
            assert_eq!(f(keep), PathBuf::from(keep), "{keep} must not become a bare `md`");
        }
        // Ordinary names are never touched, inner dots survive.
        for keep in ["Node.js.md", "Ordinary Note.md", "v1.2.3.md", "الكيماويات.md"] {
            assert_eq!(f(keep), PathBuf::from(keep));
        }
    }

    #[test]
    fn a_long_non_ascii_title_truncates_without_panicking() {
        // PJ-409: `String::truncate` panics off a char boundary, and an import is exactly where
        // non-ASCII titles arrive. 300 Arabic chars = 600 bytes, so the cut lands mid-codepoint
        // on the old code.
        let long = "ا".repeat(300);
        let out = sanitize_filename(&long);
        assert!(out.len() <= 200);
        assert!(!out.is_empty());
        assert!(std::str::from_utf8(out.as_bytes()).is_ok(), "must remain valid UTF-8");
    }
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

/// Pre-compiled regex patterns for HTML→Markdown conversion (compiled once via OnceLock).
struct HtmlPatterns {
    heading: regex::Regex,
    bold: regex::Regex,
    italic: regex::Regex,
    link: regex::Regex,
    image: regex::Regex,
    li: regex::Regex,
    list_wrap: regex::Regex,
    code_block: regex::Regex,
    inline_code: regex::Regex,
    blockquote: regex::Regex,
    hr: regex::Regex,
    br: regex::Regex,
    p: regex::Regex,
    div: regex::Regex,
    strip: regex::Regex,
}

fn html_patterns() -> &'static HtmlPatterns {
    use std::sync::OnceLock;
    static P: OnceLock<HtmlPatterns> = OnceLock::new();
    P.get_or_init(|| HtmlPatterns {
        heading: regex::Regex::new(r"<h([1-6])[^>]*>(.*?)</h\1>").unwrap(),
        bold: regex::Regex::new(r"<(b|strong)[^>]*>(.*?)</\1>").unwrap(),
        italic: regex::Regex::new(r"<(i|em)[^>]*>(.*?)</\1>").unwrap(),
        link: regex::Regex::new(r#"<a[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#).unwrap(),
        image: regex::Regex::new(r#"<img[^>]*src="([^"]*)"[^>]*/?\s*>"#).unwrap(),
        li: regex::Regex::new(r"<li[^>]*>(.*?)</li>").unwrap(),
        list_wrap: regex::Regex::new(r"</?[ou]l[^>]*>").unwrap(),
        code_block: regex::Regex::new(r"<pre[^>]*><code[^>]*>([\s\S]*?)</code></pre>").unwrap(),
        inline_code: regex::Regex::new(r"<code[^>]*>(.*?)</code>").unwrap(),
        blockquote: regex::Regex::new(r"<blockquote[^>]*>(.*?)</blockquote>").unwrap(),
        hr: regex::Regex::new(r"<hr[^>]*/?\s*>").unwrap(),
        br: regex::Regex::new(r"<br[^>]*/?\s*>").unwrap(),
        p: regex::Regex::new(r"<p[^>]*>(.*?)</p>").unwrap(),
        div: regex::Regex::new(r"<div[^>]*>(.*?)</div>").unwrap(),
        strip: regex::Regex::new(r"<[^>]+>").unwrap(),
    })
}

fn convert_html_tags(html: &str) -> String {
    let p = html_patterns();
    let mut md = html.to_string();

    md = p.heading.replace_all(&md, |caps: &regex::Captures| {
        let level: usize = caps[1].parse().unwrap_or(1);
        let text = p.strip.replace_all(&caps[2], "");
        format!("{} {}", "#".repeat(level), text.trim())
    }).to_string();
    md = p.bold.replace_all(&md, "**$2**").to_string();
    md = p.italic.replace_all(&md, "*$2*").to_string();
    md = p.link.replace_all(&md, "[$2]($1)").to_string();
    md = p.image.replace_all(&md, "![]($1)").to_string();
    md = p.li.replace_all(&md, "- $1").to_string();
    md = p.list_wrap.replace_all(&md, "").to_string();
    md = p.code_block.replace_all(&md, "```\n$1\n```").to_string();
    md = p.inline_code.replace_all(&md, "`$1`").to_string();
    md = p.blockquote.replace_all(&md, "> $1").to_string();
    md = p.hr.replace_all(&md, "\n---\n").to_string();
    md = p.br.replace_all(&md, "\n").to_string();
    md = p.p.replace_all(&md, "$1\n\n").to_string();
    md = p.div.replace_all(&md, "$1\n").to_string();
    md = p.strip.replace_all(&md, "").to_string();

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

                    match crate::write_gate::gate_write(&dest_path, &enriched, None, "import") {
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
