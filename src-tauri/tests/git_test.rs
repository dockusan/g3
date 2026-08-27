#[path = "support.rs"]
mod support;

use tauri_app_lib::git;
use tauri_app_lib::model::SideStatus;

#[test]
fn lists_the_conflicted_file() {
    let fx = support::modify_modify_conflict();
    let conflicts = git::list_conflicts(&fx.repo).unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].path, "file.txt");
    assert_eq!(conflicts[0].ours_status, SideStatus::Modified);
    assert_eq!(conflicts[0].theirs_status, SideStatus::Modified);
    assert!(!conflicts[0].is_binary);
}

#[test]
fn reads_all_three_stages() {
    let fx = support::modify_modify_conflict();
    let stages = git::read_stages(&fx.repo, "file.txt").unwrap();
    assert!(stages.base.is_some());
    assert!(stages.ours.as_ref().unwrap().contains("MAIN"));
    assert!(stages.theirs.as_ref().unwrap().contains("FEATURE"));
}

#[test]
fn flags_conflict_as_binary_when_only_theirs_side_is_binary() {
    let fx = support::binary_modify_modify_conflict();
    let conflicts = git::list_conflicts(&fx.repo).unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].path, "file.txt");
    // "ours" (main) is text, "theirs" (feature) is binary; is_binary must be true
    // because either side being binary should mark the conflict as binary.
    assert!(conflicts[0].is_binary);
    assert!(git::is_binary_conflict(&fx.repo, "file.txt").unwrap());
}

#[test]
fn discovers_repo_from_subdirectory() {
    let fx = support::modify_modify_conflict();
    let sub = fx.dir.path().join("nested");
    std::fs::create_dir_all(&sub).unwrap();
    let repo = git::discover_repo(&sub).unwrap();
    assert!(repo.path().exists());
}
