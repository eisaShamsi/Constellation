mod ai;
mod bases;
mod universe;
mod vaults;
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(watcher::WatcherState::new())
        .manage(universe::UniverseState::new())
        .invoke_handler(tauri::generate_handler![
            ai::ai_send_message,
            ai::ai_validate_connection,
            ai::ai_list_models,
            vaults::list_vaults,
            vaults::add_vault,
            vaults::remove_vault,
            vaults::read_vault_tree,
            vaults::read_note,
            vaults::get_note_headings,
            vaults::write_note,
            vaults::pick_folder,
            vaults::get_all_vault_stats,
            vaults::search_stars,
            vaults::search_by_property,
            vaults::create_note,
            vaults::create_folder,
            vaults::rename_item,
            vaults::delete_item,
            vaults::resolve_wikilink,
            vaults::resolve_wikilink_cross_vault,
            vaults::read_obsidian_appearance,
            vaults::scan_vault_links,
            vaults::scan_unlinked_mentions,
            vaults::scan_vault_tags,
            vaults::collect_vault_notes,
            vaults::get_daily_note_path,
            vaults::update_links_on_rename,
            vaults::read_note_preview,
            vaults::save_clipboard_image,
            vaults::export_note_html,
            vaults::move_to_trash,
            vaults::scan_vault_index,
            watcher::watch_vault,
            watcher::unwatch_vault,
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
            universe::resolve_universe_vaults,
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
