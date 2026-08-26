pub mod model;
pub mod conflict;
pub mod diff;
pub mod document;
pub mod git;
pub mod writer;
pub mod commands;
pub mod cli;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // CLI-invoked: `g3 [path]` / `g3 --repo <path>`, else the process cwd.
    let initial = std::env::current_dir().ok().map(|cwd| {
        let args: Vec<String> = std::env::args().collect();
        cli::resolve_repo_path(&args, &cwd)
    });

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
