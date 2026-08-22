//! Knowledge Strata — Cognitive Engine Phase 2.
//!
//! Auto-classifies every note into an 8-level hierarchy using graph topology.
//! No AI, no manual tagging. Pure structural signals: word count, link count,
//! link types, and centrality proxy.
//!
//! 8-Level Hierarchy:
//!   1 Datum          — ≤50 words, 0 links, raw fact
//!   2 Information    — 50–200 words, 0–1 links, single topic
//!   3 Proposition    — 200+ words or 2+ links
//!   4 Concept        — links 3+ notes, has `generalizes` links
//!   5 Principle      — links 3+ concepts, has `causes` or `supports` links
//!   6 Theory         — MOC (8+ outgoing links), many `part-of` inbound
//!   7 Paradigm       — referenced by 3+ high-stratum notes, high centrality
//!   8 Worldview      — highest centrality, deepest `derives-from` chain root

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tauri::Manager;

/// MIG-005 §2: read `note_aliases` into an in-memory `alias_lower → path`
/// map. See `map.rs::load_alias_map` for the canonical comment — same
/// shape repeated here per Option A's per-surface discipline (no shared
/// helper module). Failure paths degrade to an empty map (pre-MIG-005
/// alias-blind behavior).
fn load_alias_map(app: &tauri::AppHandle) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let state = app.state::<crate::search::SearchState>();
    let guard = match state.db.lock() {
        Ok(g) => g,
        Err(_) => return map,
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => return map,
    };
    let mut stmt = match conn
        .prepare("SELECT alias_lower, path FROM note_aliases ORDER BY path")
    {
        Ok(s) => s,
        Err(_) => return map,
    };
    let rows = match stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    }) {
        Ok(rs) => rs,
        Err(_) => return map,
    };
    for r in rows.flatten() {
        map.entry(r.0).or_insert(r.1);
    }
    map
}

/// Per-note stratum result returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct NoteStratum {
    pub note_path: String,
    pub note_name: String,
    pub stratum: u8,
    pub word_count: usize,
    pub outgoing_links: usize,
    pub inbound_links: usize,
}

/// Internal note record used during computation.
struct NoteRecord {
    path: String,
    name: String,
    word_count: usize,
    outgoing: Vec<String>,           // target note names
    outgoing_types: HashSet<String>, // link types used in outgoing links
    inbound_count: usize,
    inbound_sources: HashSet<String>, // unique source note names
}

/// Compute the knowledge stratum for every note in a library.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn compute_note_strata(
    app: tauri::AppHandle,
    library_path: String,
    _library_name: String,
) -> Result<Vec<NoteStratum>, String> {
    // Validate access
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]")
        .map_err(|e| e.to_string())?;

    // Phase 1: Scan all notes — collect word count + outgoing links + link types
    // MIG-111 B4 — whose vocabulary is this? The universe that OWNS `library_path`.
    // The access gate above accepts Linked-Universe libraries, and the Sky
    // enrichment loop calls this for EVERY federated library — so the active
    // vocabulary mislabeled a linked library's typed links. One owner-resolved
    // registry for the whole walk (the B3 one-value-from-the-top rule).
    let reg = crate::link_types::registry_for_owner_of(&app, &library_path)?;
    let mut notes: HashMap<String, NoteRecord> = HashMap::new();
    scan_notes_recursive(&reg, Path::new(&library_path), &re, &mut notes);

    // Phase 2: Build inbound map
    let note_names: HashSet<String> = notes.keys().cloned().collect();

    // MIG-005 §2: load alias map + path→name lookup so renamed targets
    // count toward the renamed note. 3-tier resolution mirrors
    // cache.rs::read_sky_links_raw (MIG-004 §8).
    let alias_to_path = load_alias_map(&app);
    let path_to_name: HashMap<String, String> = notes
        .values()
        .map(|n| (n.path.clone(), n.name.to_lowercase()))
        .collect();

    // Collect inbound data first (can't borrow notes mutably while iterating)
    let mut inbound_data: HashMap<String, (usize, HashSet<String>)> = HashMap::new();
    for record in notes.values() {
        for target_name in &record.outgoing {
            let target_lower = target_name.to_lowercase();
            let canonical = if note_names.contains(&target_lower) {
                target_lower
            } else if let Some(canonical_path) = alias_to_path.get(&target_lower) {
                match path_to_name.get(canonical_path) {
                    Some(n) => n.clone(),
                    None => continue,
                }
            } else {
                continue;
            };
            let entry = inbound_data.entry(canonical).or_insert((0, HashSet::new()));
            entry.0 += 1;
            entry.1.insert(record.name.clone());
        }
    }
    for (name, (count, sources)) in inbound_data {
        if let Some(record) = notes.get_mut(&name) {
            record.inbound_count += count;
            record.inbound_sources = sources;
        }
    }

    // Phase 3: Compute stratum for each note
    let results: Vec<NoteStratum> = notes.values().map(|n| {
        let stratum = compute_stratum(n);
        NoteStratum {
            note_path: n.path.clone(),
            note_name: n.name.clone(),
            stratum,
            word_count: n.word_count,
            outgoing_links: n.outgoing.len(),
            inbound_links: n.inbound_count,
        }
    }).collect();

    Ok(results)
}

/// Recursively scan a directory for .md files, building NoteRecords.
fn scan_notes_recursive(
    reg: &crate::link_types::LinkTypeRegistry,
    dir: &Path,
    re: &regex::Regex,
    notes: &mut HashMap<String, NoteRecord>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    // MIG-067 §D — registry membership (8 typed acts + custom + the null
    // `associative`). **MIG-111 §1.2/A5: the registry now arrives as a PARAMETER.** It used to
    // be re-read from the process-global once per directory, which meant a walk spanning a
    // vocabulary change could classify the first half of a library under one vocabulary and the
    // second half under another — LL-047's "which vocabulary?" is a question about WHEN.
    // One value, resolved once by the caller, used for the whole walk.

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden directories/files
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            scan_notes_recursive(reg, &path, re, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // MIG-008 Step 3: stratum results use the frontmatter title
                // as the note's display name so panels reading
                // `compute_note_strata` output show "Apple Tree Fruit"
                // instead of "20260426T140737Z_NOTE_E561".
                let note_name = crate::libraries::note_display_name(&path, Some(&content));

                // Strip YAML frontmatter for word count
                let body = strip_frontmatter(&content);
                let word_count = count_words(body);

                // PJ-065 — a wikilink under a structural (parent/TOC) frontmatter key
                // is not a cognitive outgoing link. `fm_len` is the frontmatter prefix
                // length (body is a subslice of content); the byte-offset guard skips
                // only the frontmatter occurrence, so a body link to the same note is
                // still counted. Populated since §5 (empty only if the lane is ever un-registered).
                let fm_len = body.as_ptr() as usize - content.as_ptr() as usize;
                let struct_fm = crate::link_types::structural_frontmatter_targets(reg, &content[..fm_len]);

                // Parse outgoing wikilinks + link types
                let mut outgoing: Vec<String> = Vec::new();
                let mut outgoing_types: HashSet<String> = HashSet::new();

                for cap in re.captures_iter(&content) {
                    // MIG-067 — predicate-first aware ([[type::target]] AND the
                    // legacy [[target|type]] / [[target|type:X]] forms).
                    let (target, link_type) = crate::link_types::resolve_wikilink_type(
                        &reg, &cap[1], cap.get(2).map(|m| m.as_str()), true,
                    );
                    let tl = target.to_lowercase();
                    if cap.get(0).map_or(false, |m| m.start() < fm_len) && struct_fm.contains(&tl) {
                        continue; // PJ-065 — structural-keyed frontmatter link
                    }
                    outgoing.push(tl);
                    if let Some(lt) = link_type {
                        outgoing_types.insert(lt);
                    }
                }

                notes.insert(note_name.to_lowercase(), NoteRecord {
                    path: path.to_string_lossy().to_string(),
                    name: note_name,
                    word_count,
                    outgoing,
                    outgoing_types,
                    inbound_count: 0,
                    inbound_sources: HashSet::new(),
                });
            }
        }
    }
}

/// Compute the stratum (1–8) for a single note based on structural signals.
fn compute_stratum(n: &NoteRecord) -> u8 {
    // Base from word count
    let base: u8 = if n.word_count <= 50 {
        1
    } else if n.word_count <= 200 {
        2
    } else {
        3
    };

    let mut bonus: u8 = 0;

    // Link count bonus
    if n.outgoing.len() >= 3 {
        bonus += 1;
    }
    if n.inbound_count >= 5 {
        bonus += 1;
    }

    // Semantic link type bonus
    if n.outgoing_types.contains("generalizes") {
        bonus += 1;
    }
    if n.outgoing_types.contains("causes") || n.outgoing_types.contains("supports") {
        bonus += 1;
    }

    // Centrality proxy: inbound from 3+ unique sources
    if n.inbound_sources.len() >= 3 {
        bonus += 1;
    }

    // Clamp to 1–8
    (base + bonus).clamp(1, 8)
}

/// Strip YAML frontmatter (--- delimited) from content, return body.
/// Also exposed as `strip_frontmatter_pub` for use by other modules.
pub fn strip_frontmatter_pub(content: &str) -> &str { strip_frontmatter(content) }
fn strip_frontmatter(content: &str) -> &str {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            return &content[end + 7..]; // skip past closing ---\n
        }
    }
    content
}

/// Count words in text (split on whitespace, filter empty).
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}
