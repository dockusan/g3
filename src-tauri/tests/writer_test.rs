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

#[test]
fn rejects_absolute_path() {
    let fx = support::modify_modify_conflict();
    let result = writer::save_resolution(&fx.repo, "/etc/passwd", "x");
    assert!(result.is_err());

    // The legitimate target file in the fixture repo was untouched.
    let on_disk = std::fs::read_to_string(fx.dir.path().join("file.txt")).unwrap();
    assert_ne!(on_disk, "x");
}

#[test]
fn rejects_path_with_parent_dir_component() {
    let fx = support::modify_modify_conflict();
    let result = writer::save_resolution(&fx.repo, "../outside.txt", "x");
    assert!(result.is_err());

    // Nothing escaped the temp dir into its parent.
    let escaped_path = fx.dir.path().parent().unwrap().join("outside.txt");
    assert!(!escaped_path.exists());
}
