use crate::{document, git, model::ConflictDocument, model::ConflictFile, writer};
use std::path::PathBuf;
use std::sync::Mutex;

// Holds the repo path chosen at startup (cwd) or via the folder picker.
pub struct AppState {
    pub repo_path: Mutex<Option<PathBuf>>,
}

fn open_repo(state: &AppState) -> Result<git2::Repository, String> {
    let guard = state.repo_path.lock().unwrap();
    let path = guard.as_ref().ok_or("no repository selected")?;
    git::discover_repo(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_repo(state: tauri::State<AppState>, path: String) -> Result<Vec<ConflictFile>, String> {
    *state.repo_path.lock().unwrap() = Some(PathBuf::from(&path));
    list_conflicts(state)
}

#[tauri::command]
pub fn list_conflicts(state: tauri::State<AppState>) -> Result<Vec<ConflictFile>, String> {
    let repo = open_repo(&state)?;
    git::list_conflicts(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_conflict(state: tauri::State<AppState>, path: String) -> Result<ConflictDocument, String> {
    let repo = open_repo(&state)?;
    document::load(&repo, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_resolution(
    state: tauri::State<AppState>,
    path: String,
    content: String,
) -> Result<Vec<ConflictFile>, String> {
    let repo = open_repo(&state)?;
    writer::save_resolution(&repo, &path, &content).map_err(|e| e.to_string())?;
    git::list_conflicts(&repo).map_err(|e| e.to_string())
}
