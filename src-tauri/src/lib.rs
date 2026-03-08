mod ai;
mod vaults;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            vaults::search_stars
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
