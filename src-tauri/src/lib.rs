pub mod model;
pub mod conflict;
pub mod diff;
pub mod document;
pub mod git;
pub mod writer;
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // CLI-invoked: use current working directory as the initial repo candidate.
    let initial = std::env::current_dir().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::AppState { repo_path: std::sync::Mutex::new(initial) })
        .invoke_handler(tauri::generate_handler![
            commands::set_repo,
            commands::list_conflicts,
            commands::load_conflict,
            commands::save_resolution,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
