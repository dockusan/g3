use std::path::{Path, PathBuf};
use tauri_app_lib::cli::resolve_repo_path;

fn args(rest: &[&str]) -> Vec<String> {
    std::iter::once("g3".to_string())
        .chain(rest.iter().map(|s| s.to_string()))
        .collect()
}

#[test]
fn no_args_uses_cwd() {
    let cwd = Path::new("/work/my-repo");
    assert_eq!(
        resolve_repo_path(&args(&[]), cwd),
        PathBuf::from("/work/my-repo")
    );
}

#[test]
fn positional_path_is_used() {
    let cwd = Path::new("/work");
    assert_eq!(
        resolve_repo_path(&args(&["/tmp/conflicted"]), cwd),
        PathBuf::from("/tmp/conflicted")
    );
}

#[test]
fn relative_positional_is_joined_to_cwd() {
    let cwd = Path::new("/work");
    assert_eq!(
        resolve_repo_path(&args(&["../other"]), cwd),
        cwd.join("../other")
    );
}

#[test]
fn repo_flag_is_used() {
    let cwd = Path::new("/work");
    assert_eq!(
        resolve_repo_path(&args(&["--repo", "/tmp/conflicted"]), cwd),
        PathBuf::from("/tmp/conflicted")
    );
}

#[test]
fn relative_repo_flag_is_joined_to_cwd() {
    let cwd = Path::new("/work");
    assert_eq!(
        resolve_repo_path(&args(&["--repo", "nested"]), cwd),
        cwd.join("nested")
    );
}
