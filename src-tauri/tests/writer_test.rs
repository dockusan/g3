#[path = "support.rs"]
mod support;

use tauri_app_lib::{git, writer};

#[test]
fn save_writes_file_and_marks_resolved() {
    let fx = support::modify_modify_conflict();
    let resolved = "line1\nRESOLVED\nline3\n";
    writer::save_resolution(&fx.repo, "file.txt", resolved).unwrap();

    // File on disk has the resolved content.
    let on_disk = std::fs::read_to_string(fx.dir.path().join("file.txt")).unwrap();
    assert_eq!(on_disk, resolved);

    // No longer listed as a conflict.
    let conflicts = git::list_conflicts(&fx.repo).unwrap();
    assert!(conflicts.is_empty());
}
