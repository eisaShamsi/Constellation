mod ai;
mod arabic;
mod bases;
mod canvas;
mod boot_bundle;
mod cache;
mod canonical;
mod embeddings;
mod embeds;
mod dataview;
mod file_kinds;
mod fts5_tokenizer;
mod inspector360;
mod importers;
mod libraries;
mod lens;
mod lenses;
mod lexicon;
mod search;
mod map;
mod maturity;
mod perf_trace;
mod provenance;
mod review;
mod strata;
mod tension;
mod trails;
mod tasks;
mod universe;
mod watcher;

use tauri::{Emitter, Manager};

#[tauri::command]
fn open_second_screen(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("second-screen") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        // Force WebView2 to repaint after showing a hidden window
        let _ = win.eval("void(0)");
    }
    Ok(())
}

#[tauri::command]
fn constellation_show_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/* ------------------------------------------------------------------ */
/*  Monitor / display detection                                        */
/* ------------------------------------------------------------------ */

#[derive(serde::Serialize, Clone)]
pub struct MonitorInfo {
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[tauri::command]
fn list_monitors(app: tauri::AppHandle) -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();
    let primary = app.primary_monitor().ok().flatten();
    let primary_name = primary.as_ref().and_then(|m| m.name().map(|n| n.to_string()));

    if let Ok(available) = app.available_monitors() {
        for mon in available {
            let pos = mon.position();
            let size = mon.size();
            let name = mon.name().map(|n| n.to_string());
            let is_primary = match (&name, &primary_name) {
                (Some(n), Some(pn)) => n == pn,
                _ => false,
            };
            monitors.push(MonitorInfo {
                name,
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
                scale_factor: mon.scale_factor(),
                is_primary,
            });
        }
    }
    monitors
}

/// Open the second screen, auto-positioning on a secondary monitor if available.
#[tauri::command]
fn open_second_screen_on_monitor(app: tauri::AppHandle) -> Result<(), String> {
    let win = app.get_webview_window("second-screen")
        .ok_or_else(|| "second-screen window not found".to_string())?;

    let monitors: Vec<_> = app.available_monitors().map_err(|e| e.to_string())?
        .into_iter().collect();
    let primary = app.primary_monitor().ok().flatten();
    let primary_name = primary.as_ref().and_then(|m| m.name().map(|n| n.to_string()));

    // Strategy 1: Find monitor by name (different from primary)
    let secondary = monitors.iter().find(|m| {
        match (m.name().map(|n| n.to_string()), &primary_name) {
            (Some(n), Some(pn)) => n != *pn,
            _ => false, // Don't assume unnamed monitors are secondary
        }
    }).or_else(|| {
        // Strategy 2: Pick the monitor NOT at origin (0,0) — primary is usually there
        monitors.iter().find(|m| {
            let pos = m.position();
            pos.x != 0 || pos.y != 0
        })
    }).or_else(|| {
        // Strategy 3: If all monitors are at (0,0), pick the second one
        if monitors.len() > 1 { Some(&monitors[1]) } else { None }
    });

    if let Some(mon) = secondary {
        let pos = mon.position();
        let size = mon.size();
        let win_w = (size.width as f64 * 0.8) as u32;
        let win_h = (size.height as f64 * 0.8) as u32;
        let win_x = pos.x + ((size.width - win_w) / 2) as i32;
        let win_y = pos.y + ((size.height - win_h) / 2) as i32;

        use tauri::PhysicalPosition;
        use tauri::PhysicalSize;
        let _ = win.set_position(PhysicalPosition::new(win_x, win_y));
        let _ = win.set_size(PhysicalSize::new(win_w, win_h));
    }

    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    let _ = win.eval("void(0)");
    Ok(())
}

#[tauri::command]
fn close_second_screen(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("second-screen") {
        win.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn is_second_screen_open(app: tauri::AppHandle) -> bool {
    if let Some(win) = app.get_webview_window("second-screen") {
        win.is_visible().unwrap_or(false)
    } else {
        false
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Install panic hook to log crashes before the process exits
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("[CONSTELLATION PANIC] {}", info);
        eprintln!("{}", msg);
        // Also write to a crash log file next to the executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let log_path = dir.join("constellation-crash.log");
                let _ = std::fs::write(&log_path, &msg);
            }
        }
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(watcher::WatcherState::new())
        .manage(universe::UniverseState::new())
        .manage(search::SearchState::new())
        .manage(embeddings::EmbeddingState { engine: std::sync::Mutex::new(None) })
        .invoke_handler({
            // Round 6 diagnostic (2026-04-19) — IPC arrival tracer.
            //
            // Round 5 (scan_library_tags → async) + DashboardView gate
            // (`{#if false}`) + JS heartbeat (max_gap = 112 ms) all
            // falsified their respective hypotheses: Criterion 2 still
            // fails at ~19 s with core_queue_ms ≈ 18.6 s, JS is alive,
            // and no single frontend gate moves the needle.
            //
            // The remaining unknown is what happens between JS
            // `postMessage` and the Rust command body entering. We wrap
            // `generate_handler!` in a closure that stamps a Unix-ms
            // timestamp on every dispatch into `perf_trace::TRACE_LOG`,
            // then forwards to the inner handler unchanged. The log is
            // fetched by the frontend at `boot:hydrated` via
            // `get_perf_trace_log` and bundled into the boot-perf JSON.
            //
            // Overhead is a single Mutex lock per command + a
            // `(String, u64)` push; negligible vs. any actual IPC cost.
            //
            // The `Box<dyn Fn(...)>` annotation pins the runtime to `Wry`
            // at the binding site — without it, the macro's `R: Runtime`
            // generic is unresolvable in a `let` binding (it only infers
            // when passed directly to `invoke_handler`).
            let inner: Box<dyn Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static> = Box::new(tauri::generate_handler![
            ai::ai_send_message,
            ai::ai_validate_connection,
            ai::ai_list_models,
            libraries::list_libraries,
            libraries::add_library,
            libraries::remove_library,
            libraries::read_library_tree,
            libraries::read_note,
            libraries::get_note_headings,
            libraries::write_note,
            libraries::pick_folder,
            libraries::create_new_library,
            libraries::get_all_library_stats,
            libraries::search_stars,
            libraries::search_by_property,
            libraries::create_note,
            libraries::create_folder,
            libraries::rename_item,
            libraries::move_item,
            libraries::delete_item,
            libraries::resolve_wikilink,
            libraries::resolve_wikilink_cross_library,
            libraries::read_library_appearance,
            libraries::scan_library_links,
            libraries::scan_unlinked_mentions,
            libraries::scan_library_tags,
            libraries::collect_library_notes,
            libraries::collect_library_notes_with_metadata,
            libraries::get_daily_note_path,
            libraries::quick_capture,
            libraries::update_links_on_rename,
            libraries::read_note_preview,
            libraries::save_clipboard_image,
            libraries::resolve_embed_image,
            embeds::resolve_embed,
            embeds::read_vault_config_cmd,
            embeds::invalidate_vault_index_cmd,
            libraries::scan_note_stages,
            strata::compute_note_strata,
            maturity::compute_note_maturity,
            tension::detect_tensions,
            provenance::get_provenance_chain,
            provenance::compute_note_origins,
            review::get_due_notes,
            review::mark_reviewed,
            review::snooze_note,
            review::dismiss_note,
            review::record_note_visit,
            trails::list_trails,
            trails::read_trail,
            canvas::list_canvases,
            inspector360::get_360_view,
            lens::constellation_lens_centrality,
            lens::constellation_lens_tag_edges,
            boot_bundle::constellation_boot_bundle,
            cache::cache_boot_snapshot,
            cache::cache_boot_snapshot_core,
            cache::cache_boot_snapshot_graph,
            cache::cache_is_populated,
            cache::cache_reconcile,
            cache::write_boot_perf_report,
            cache::read_boot_perf_report,
            search::constellation_search_init,
            search::constellation_search,
            search::constellation_search_reindex,
            search::constellation_search_store_embedding,
            search::constellation_search_similar,
            search::constellation_search_universal,
            search::constellation_search_link_counts,
            search::constellation_link_stats,
            search::constellation_link_traverse,
            search::constellation_debug_link_state,
            search::constellation_link_dormant,
            search::constellation_link_decay,
            search::constellation_link_set_confidence,
            search::constellation_link_archive,
            search::constellation_formulation_analysis,
            embeddings::constellation_init_embeddings,
            embeddings::constellation_embed_text,
            embeddings::constellation_embed_notes,
            embeddings::constellation_embedding_status,
            map::constellation_map_data,
            map::constellation_map_universe,
            canvas::read_canvas,
            canvas::write_canvas,
            canvas::create_canvas,
            lenses::list_lenses,
            lenses::save_lenses,
            lenses::apply_lens,
            libraries::export_note_html,
            libraries::move_to_trash,
            libraries::scan_library_index,
            libraries::read_index_entries,
            libraries::read_term_mentions,
            watcher::watch_library,
            watcher::unwatch_library,
            bases::parse_base_file,
            bases::query_base,
            bases::save_base_file,
            bases::update_note_property,
            bases::create_base,
            bases::list_workspace_bases,
            bases::create_workspace_base,
            bases::save_workspace_base,
            bases::delete_workspace_base,
            bases::parse_workspace_base,
            universe::list_universes,
            universe::create_universe,
            universe::set_active_universe,
            universe::get_active_universe_path,
            universe::remove_universe_from_registry,
            universe::check_migration_needed,
            universe::add_child_universe,
            universe::remove_child_universe,
            universe::resolve_universe_libraries,
            universe::read_universe_settings,
            universe::save_universe_settings,
            universe::read_universe_bookmarks,
            universe::save_universe_bookmarks,
            universe::read_universe_workspaces,
            universe::save_universe_workspaces,
            universe::read_universe_property_types,
            universe::save_universe_property_types,
            universe::migrate_legacy_data,
            universe::open_existing_universe,
            universe::get_child_universes,
            universe::read_child_universe_libraries,
            universe::link_library_as_universe,
            universe::scaffold_starter_library,
            universe::get_templates_dir,
            universe::list_templates,
            universe::rename_universe,
            arabic::overrides::read_arabic_overrides,
            arabic::overrides::add_arabic_override,
            arabic::overrides::remove_arabic_override,
            arabic::overrides::reindex_arabic_overrides,
            libraries::get_file_metadata,
            libraries::notes_by_tag,
            dataview::execute_dataview_query,
            tasks::scan_library_tasks,
            tasks::scan_note_tasks,
            tasks::toggle_task,
            tasks::scan_library_note_dates,
            importers::import_pick_source,
            importers::import_preview,
            importers::import_execute,
            importers::import_with_canonical,
            file_kinds::classify_file_cmd,
            canonical::generate_canonical_name,
            canonical::canonicalize_preview,
            canonical::canonicalize_execute,
            canonical::auto_canonicalize_all,
            canonical::inject_cid_library,
            canonical::de_canonicalize_library,
            canonical::repair_external_libraries_on_startup,
            canonical::ensure_cid_cn_cmd,
            libraries::set_library_canonical_mode,
            perf_trace::get_perf_trace_log,
            perf_trace::clear_perf_trace_log,
            constellation_show_in_folder,
            open_path,
            list_monitors,
            open_second_screen,
            open_second_screen_on_monitor,
            close_second_screen,
            is_second_screen_open
            ]);
            move |invoke: tauri::ipc::Invoke<tauri::Wry>| -> bool {
                perf_trace::record(invoke.message.command());
                inner(invoke)
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "second-screen" {
                    // Intercept close on second-screen: hide instead of destroy
                    api.prevent_close();
                    let _ = window.hide();
                    // Notify frontend that screen was closed
                    let _ = window.emit("screen-hidden", ());
                } else if window.label() == "main" {
                    // Main window closing: also close the second screen
                    if let Some(second) = window.app_handle().get_webview_window("second-screen") {
                        let _ = second.destroy();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
