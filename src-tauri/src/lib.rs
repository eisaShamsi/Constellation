mod ai;
mod bases;
mod dataview;
mod importers;
mod libraries;
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
        .invoke_handler(tauri::generate_handler![
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
            libraries::export_note_html,
            libraries::move_to_trash,
            libraries::scan_library_index,
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
            universe::link_library_as_universe,
            universe::scaffold_starter_library,
            universe::get_templates_dir,
            universe::list_templates,
            libraries::get_file_metadata,
            dataview::execute_dataview_query,
            tasks::scan_library_tasks,
            tasks::scan_note_tasks,
            tasks::toggle_task,
            tasks::scan_library_note_dates,
            importers::import_pick_source,
            importers::import_preview,
            importers::import_execute,
            open_second_screen,
            close_second_screen,
            is_second_screen_open
        ])
        .on_window_event(|window, event| {
            // Intercept close on second-screen: hide instead of destroy
            if window.label() == "second-screen" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                    // Notify frontend that screen was closed
                    let _ = window.emit("screen-hidden", ());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
