mod ai;
mod vaults;
mod watcher;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(watcher::WatcherState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            ai::ai_send_message,
            ai::ai_validate_connection,
            ai::ai_list_models,
            vaults::list_vaults,
            vaults::add_vault,
            vaults::remove_vault,
            vaults::read_vault_tree,
            vaults::read_note,
            vaults::write_note,
            vaults::pick_folder,
            vaults::get_all_vault_stats,
            vaults::search_stars,
            vaults::create_note,
            vaults::create_folder,
            vaults::rename_item,
            vaults::delete_item,
            vaults::resolve_wikilink,
            vaults::read_obsidian_appearance,
            vaults::scan_vault_links,
            vaults::scan_vault_tags,
            vaults::collect_vault_notes,
            vaults::get_daily_note_path,
            vaults::update_links_on_rename,
            vaults::read_note_preview,
            vaults::save_clipboard_image,
            vaults::export_note_html,
            vaults::move_to_trash,
            watcher::watch_vault,
            watcher::unwatch_vault
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
