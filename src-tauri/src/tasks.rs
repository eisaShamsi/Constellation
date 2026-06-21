use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::libraries::validate_path_in_any_library;

// ─── Types ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub text: String,
    pub completed: bool,
    pub file_path: String,
    pub file_name: String,
    pub library_name: String,
    pub library_path: String,
    pub line_number: usize, // 1-indexed
    pub due_date: Option<String>,
    pub priority: Option<String>, // "high", "medium", "low"
    pub tags: Vec<String>,
    pub created_date: Option<String>,
    pub done_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScanResult {
    pub tasks: Vec<TaskItem>,
    pub total_count: usize,
    pub scan_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDateEntry {
    pub file_path: String,
    pub file_name: String,
    pub date: String,        // YYYY-MM-DD
    pub date_source: String, // "modified", "created", "frontmatter"
    pub library_name: String, // MIG-082 §A.1 — so a calendar dot can open in the right library
    pub is_daily: bool,       // MIG-082 §A.1 — true if THIS file is the daily note FOR this date
                              // (matches get_daily_note_path: daily_folder + format(date).md)
}

/// MIG-082 §A.1 — is `path` the daily note FOR `date`? Mirrors get_daily_note_path's
/// filename construction (the daily folder + `dailyNoteFormat` applied to a midnight datetime),
/// so the truth lives in one place and the frontend never re-implements strftime.
fn is_daily_note_for(path: &Path, date: &str, daily_dir: &Path, daily_format: &str) -> bool {
    let nd = match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return false,
    };
    let dt = match nd.and_hms_opt(0, 0, 0) {
        Some(d) => d,
        None => return false,
    };
    let stem = dt.format(daily_format).to_string();
    path == daily_dir.join(format!("{}.md", stem))
}

// ─── Task Parsing ───

/// Extract due date from task text.
/// Supports: 📅 2026-03-14, due:: 2026-03-14, [due:: 2026-03-14]
fn extract_due_date(text: &str) -> Option<String> {
    // Emoji format: 📅 YYYY-MM-DD
    if let Some(pos) = text.find('\u{1F4C5}') {
        let after = &text[pos + '\u{1F4C5}'.len_utf8()..].trim_start();
        if after.len() >= 10 {
            let date_str = &after[..10];
            if is_valid_date(date_str) {
                return Some(date_str.to_string());
            }
        }
    }
    // Inline field: [due:: YYYY-MM-DD] or due:: YYYY-MM-DD
    let due_patterns = ["[due:: ", "due:: ", "[due::"];
    for pat in &due_patterns {
        if let Some(pos) = text.to_lowercase().find(&pat.to_lowercase()) {
            let after = &text[pos + pat.len()..].trim_start();
            if after.len() >= 10 {
                let date_str = &after[..10];
                if is_valid_date(date_str) {
                    return Some(date_str.to_string());
                }
            }
        }
    }
    None
}

/// Extract priority from task text.
/// Supports: ⏫ (high), 🔼 (medium), 🔽 (low), [priority:: high/medium/low]
fn extract_priority(text: &str) -> Option<String> {
    if text.contains('\u{23EB}') {
        return Some("high".to_string());
    }
    if text.contains('\u{1F53C}') {
        return Some("medium".to_string());
    }
    if text.contains('\u{1F53D}') {
        return Some("low".to_string());
    }
    // Inline field: [priority:: high]
    let lower = text.to_lowercase();
    if let Some(pos) = lower.find("priority::") {
        let after = lower[pos + 10..].trim_start();
        let val = after.split(']').next().unwrap_or(after).split_whitespace().next().unwrap_or("");
        match val {
            "high" => return Some("high".to_string()),
            "medium" => return Some("medium".to_string()),
            "low" => return Some("low".to_string()),
            _ => {}
        }
    }
    None
}

/// Extract tags from task text (#tag patterns).
fn extract_tags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let re = regex::Regex::new(r"(?:^|\s)#([\w\p{Arabic}/\-]+)").unwrap();
    for cap in re.captures_iter(text) {
        tags.push(format!("#{}", &cap[1]));
    }
    tags
}

/// Extract done date: ✅ YYYY-MM-DD or [completion:: YYYY-MM-DD]
fn extract_done_date(text: &str) -> Option<String> {
    // ✅ YYYY-MM-DD
    if let Some(pos) = text.find('\u{2705}') {
        let after = &text[pos + '\u{2705}'.len_utf8()..].trim_start();
        if after.len() >= 10 {
            let date_str = &after[..10];
            if is_valid_date(date_str) {
                return Some(date_str.to_string());
            }
        }
    }
    // [completion:: YYYY-MM-DD]
    let patterns = ["[completion:: ", "completion:: "];
    for pat in &patterns {
        if let Some(pos) = text.to_lowercase().find(&pat.to_lowercase()) {
            let after = &text[pos + pat.len()..].trim_start();
            if after.len() >= 10 {
                let date_str = &after[..10];
                if is_valid_date(date_str) {
                    return Some(date_str.to_string());
                }
            }
        }
    }
    None
}

/// Extract created date: ➕ YYYY-MM-DD or [created:: YYYY-MM-DD]
fn extract_created_date(text: &str) -> Option<String> {
    // ➕ YYYY-MM-DD
    if let Some(pos) = text.find('\u{2795}') {
        let after = &text[pos + '\u{2795}'.len_utf8()..].trim_start();
        if after.len() >= 10 {
            let date_str = &after[..10];
            if is_valid_date(date_str) {
                return Some(date_str.to_string());
            }
        }
    }
    // [created:: YYYY-MM-DD]
    let patterns = ["[created:: ", "created:: "];
    for pat in &patterns {
        if let Some(pos) = text.to_lowercase().find(&pat.to_lowercase()) {
            let after = &text[pos + pat.len()..].trim_start();
            if after.len() >= 10 {
                let date_str = &after[..10];
                if is_valid_date(date_str) {
                    return Some(date_str.to_string());
                }
            }
        }
    }
    None
}

fn is_valid_date(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[4] == b'-' && bytes[7] == b'-'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

/// Parse a single line for task content. Returns Some(TaskItem) if the line is a task.
fn parse_task_line(
    line: &str,
    line_number: usize,
    file_path: &str,
    file_name: &str,
    library_name: &str,
    library_path: &str,
) -> Option<TaskItem> {
    // Match: optional whitespace, -, *, or +, space, [ ] or [x] or [X]
    let trimmed = line.trim_start();
    // Use chars() for safe UTF-8 handling instead of byte indexing
    let first_char = match trimmed.chars().next() {
        Some(c) => c,
        None => return None,
    };
    if first_char != '-' && first_char != '*' && first_char != '+' {
        return None;
    }
    let after_marker = trimmed[first_char.len_utf8()..].trim_start();
    if !after_marker.starts_with("[ ]") && !after_marker.starts_with("[x]") && !after_marker.starts_with("[X]") {
        return None;
    }
    let completed = !after_marker.starts_with("[ ]");
    let text = after_marker[3..].trim_start().to_string();

    Some(TaskItem {
        due_date: extract_due_date(&text),
        priority: extract_priority(&text),
        tags: extract_tags(&text),
        created_date: extract_created_date(&text),
        done_date: extract_done_date(&text),
        text,
        completed,
        file_path: file_path.to_string(),
        file_name: file_name.to_string(),
        library_name: library_name.to_string(),
        library_path: library_path.to_string(),
        line_number,
    })
}

// ─── Recursive Scanner ───

fn scan_tasks_recursive(
    dir: &Path,
    library_name: &str,
    library_path: &str,
    tasks: &mut Vec<TaskItem>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_tasks_recursive(&path, library_name, library_path, tasks);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let file_path_str = path.to_string_lossy().to_string();
                // MIG-008 Step 4: task source labels use frontmatter title.
                let file_name = crate::libraries::note_display_name(&path, Some(&content));
                for (i, line) in content.lines().enumerate() {
                    if let Some(task) = parse_task_line(
                        line,
                        i + 1, // 1-indexed
                        &file_path_str,
                        &file_name,
                        library_name,
                        library_path,
                    ) {
                        tasks.push(task);
                    }
                }
            }
        }
    }
}

// ─── Note Date Scanner ───

fn scan_dates_recursive(
    dir: &Path,
    library_name: &str,
    daily_dir: &Path,      // MIG-082 §A.1 — the resolved daily-note folder (for is_daily)
    daily_format: &str,    // MIG-082 §A.1 — dailyNoteFormat (for is_daily)
    entries: &mut Vec<NoteDateEntry>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_dates_recursive(&path, library_name, daily_dir, daily_format, entries);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let file_path_str = path.to_string_lossy().to_string();
            // MIG-008 Step 4: scan_library_note_dates label uses frontmatter
            // title. None form here — the helper reads the file only when
            // the filename is canonical, otherwise file_stem is the title.
            let file_name = crate::libraries::note_display_name(&path, None);

            // Get modified date from filesystem
            if let Ok(meta) = fs::metadata(&path) {
                if let Ok(modified) = meta.modified() {
                    let secs = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let date = timestamp_to_date(secs);
                    let is_daily = is_daily_note_for(&path, &date, daily_dir, daily_format);
                    entries.push(NoteDateEntry {
                        file_path: file_path_str.clone(),
                        file_name: file_name.clone(),
                        date,
                        date_source: "modified".to_string(),
                        library_name: library_name.to_string(),
                        is_daily,
                    });
                }
            }

            // Check frontmatter for date property
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(props) = crate::bases::parse_frontmatter(&content) {
                    if let Some(date_val) = props.get("date") {
                        let d = date_val.trim();
                        if d.len() >= 10 && is_valid_date(&d[..10]) {
                            let date = d[..10].to_string();
                            let is_daily = is_daily_note_for(&path, &date, daily_dir, daily_format);
                            entries.push(NoteDateEntry {
                                file_path: file_path_str.clone(),
                                file_name: file_name.clone(),
                                date,
                                date_source: "frontmatter".to_string(),
                                library_name: library_name.to_string(),
                                is_daily,
                            });
                        }
                    }
                    // Also check "created" property
                    if let Some(date_val) = props.get("created") {
                        let d = date_val.trim();
                        if d.len() >= 10 && is_valid_date(&d[..10]) {
                            let date = d[..10].to_string();
                            let is_daily = is_daily_note_for(&path, &date, daily_dir, daily_format);
                            entries.push(NoteDateEntry {
                                file_path: file_path_str.clone(),
                                file_name: file_name.clone(),
                                date,
                                date_source: "frontmatter".to_string(),
                                library_name: library_name.to_string(),
                                is_daily,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn timestamp_to_date(secs: u64) -> String {
    // Convert unix timestamp to YYYY-MM-DD using chrono
    let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_default()
        .with_timezone(&chrono::Local);
    dt.format("%Y-%m-%d").to_string()
}

// ─── Tauri Commands ───

/// Scan an entire library for tasks.
#[tauri::command]
pub fn scan_library_tasks(
    app: tauri::AppHandle,
    library_path: String,
    library_name: String,
) -> Result<TaskScanResult, String> {
    validate_path_in_any_library(&app, &library_path)?;
    let start = Instant::now();
    let mut tasks = Vec::new();
    scan_tasks_recursive(Path::new(&library_path), &library_name, &library_path, &mut tasks);
    let total_count = tasks.len();
    Ok(TaskScanResult {
        tasks,
        total_count,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

/// Scan a single note file for tasks.
#[tauri::command]
pub fn scan_note_tasks(
    app: tauri::AppHandle,
    file_path: String,
    library_name: String,
    library_path: String,
) -> Result<TaskScanResult, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let start = Instant::now();
    let mut tasks = Vec::new();

    let path = Path::new(&file_path);
    if path.exists() && path.extension().and_then(|e| e.to_str()) == Some("md") {
        if let Ok(content) = fs::read_to_string(path) {
            // MIG-008 Step 4: scan_note_tasks label uses frontmatter title.
            let file_name = crate::libraries::note_display_name(path, Some(&content));
            for (i, line) in content.lines().enumerate() {
                if let Some(task) = parse_task_line(
                    line,
                    i + 1,
                    &file_path,
                    &file_name,
                    &library_name,
                    &library_path,
                ) {
                    tasks.push(task);
                }
            }
        }
    }

    let total_count = tasks.len();
    Ok(TaskScanResult {
        tasks,
        total_count,
        scan_time_ms: start.elapsed().as_millis() as u64,
    })
}

/// Toggle a task's completion status at a specific line in a file.
/// Returns the updated file content so the frontend can refresh the editor.
#[tauri::command]
pub fn toggle_task(
    app: tauri::AppHandle,
    file_path: String,
    line_number: usize,
) -> Result<String, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let path = Path::new(&file_path);

    // Safety: only .md files
    match path.extension().and_then(|e| e.to_str()) {
        Some("md") => {}
        _ => return Err("Can only modify .md files.".to_string()),
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    if line_number == 0 || line_number > lines.len() {
        return Err(format!("Line number {} out of range (1-{})", line_number, lines.len()));
    }

    let idx = line_number - 1;
    let line = &lines[idx];

    // Find and toggle the checkbox
    if let Some(bracket_pos) = line.find("[ ]") {
        let mut new_line = String::new();
        new_line.push_str(&line[..bracket_pos]);
        new_line.push_str("[x]");
        new_line.push_str(&line[bracket_pos + 3..]);
        // Add completion date if not present
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if !new_line.contains('\u{2705}') {
            new_line.push_str(&format!(" \u{2705} {}", today));
        }
        lines[idx] = new_line;
    } else if let Some(bracket_pos) = line.find("[x]").or_else(|| line.find("[X]")) {
        let mut new_line = String::new();
        new_line.push_str(&line[..bracket_pos]);
        new_line.push_str("[ ]");
        new_line.push_str(&line[bracket_pos + 3..]);
        // Remove completion date if present (✅ YYYY-MM-DD)
        let re = regex::Regex::new(r"\s*\u{2705}\s*\d{4}-\d{2}-\d{2}").unwrap();
        let new_line = re.replace(&new_line, "").to_string();
        lines[idx] = new_line;
    } else {
        return Err("No task checkbox found on this line.".to_string());
    }

    let new_content = lines.join("\n");
    // Preserve trailing newline if original had one
    let final_content = if content.ends_with('\n') {
        format!("{}\n", new_content)
    } else {
        new_content
    };

    // MIG-076 §A2 — through the WriteGate (serialized + atomic + journaled).
    crate::write_gate::gate_write(path, &final_content, None, "task_toggle")?;

    // MIG-080 §C (Debt Register E.2) — refresh the search index after the toggle.
    // Before, toggle_task wrote via the gate but NEVER reindexed → the FTS/body
    // (and any task-line tags) drifted from disk after a checkbox toggle. Reindex
    // is FREEZE-SAFE here: a checkbox toggle changes no [[links]], so index_note's
    // unchanged-edges guard (search.rs ~:4740) skips the note_links rebuild + the
    // MIG-001 trigger cascade — only the cheap note_meta/FTS refresh runs.
    // Best-effort: the disk write is the source of truth; a reindex glitch must
    // not fail the toggle (the watcher / next full reindex would catch it).
    // (bases.rs:745 pattern — resolve the library name from the path, then reindex.)
    {
        let libraries = crate::libraries::load_libraries(&app);
        let lib_name = libraries.iter().find(|v| {
            fs::canonicalize(&file_path).ok()
                .and_then(|fp| fs::canonicalize(&v.path).ok().map(|vp| fp.starts_with(vp)))
                .unwrap_or(false)
        }).map(|v| v.name.clone());
        if let Some(lib_name) = lib_name {
            use tauri::Manager;
            let search_state = app.state::<crate::search::SearchState>();
            let _ = crate::search::reindex_single_note(&search_state, &file_path, &lib_name);
        }
    }

    Ok(final_content)
}

/// Scan a library for note dates (modified + frontmatter date/created).
/// Returns a map of YYYY-MM-DD -> list of notes for that date.
#[tauri::command]
pub fn scan_library_note_dates(
    app: tauri::AppHandle,
    library_path: String,
    library_name: String,
    daily_format: Option<String>, // MIG-082 §A.1 — dailyNoteFormat; default %Y-%m-%d (so is_daily is correct)
    daily_folder: Option<String>, // MIG-082 §A.1 — dailyNoteFolder ("" = library root)
) -> Result<HashMap<String, Vec<NoteDateEntry>>, String> {
    validate_path_in_any_library(&app, &library_path)?;
    let fmt = daily_format.filter(|s| !s.is_empty()).unwrap_or_else(|| "%Y-%m-%d".to_string());
    let folder = daily_folder.unwrap_or_default();
    let daily_dir = if folder.is_empty() {
        Path::new(&library_path).to_path_buf()
    } else {
        Path::new(&library_path).join(&folder)
    };
    let mut entries = Vec::new();
    scan_dates_recursive(Path::new(&library_path), &library_name, &daily_dir, &fmt, &mut entries);

    let mut map: HashMap<String, Vec<NoteDateEntry>> = HashMap::new();
    for entry in entries {
        map.entry(entry.date.clone()).or_default().push(entry);
    }
    Ok(map)
}
