#[path = "support.rs"]
mod support;

use tauri_app_lib::{document, model::HunkKind};

#[test]
fn builds_document_with_one_conflict_hunk() {
    let fx = support::modify_modify_conflict();
    let doc = document::load(&fx.repo, "file.txt").unwrap();
    assert_eq!(doc.path, "file.txt");
    assert_eq!(doc.conflict_count, 1);
    assert!(!doc.content_hash.is_empty());
    let conflict = doc.hunks.iter().find(|h| h.kind == HunkKind::Conflict).unwrap();
    assert!(conflict.left_lines.iter().any(|l| l.contains("MAIN")));
    assert!(conflict.right_lines.iter().any(|l| l.contains("FEATURE")));
}

#[test]
fn blue_and_red_fixture_has_both() {
    let fx = support::blue_and_red_conflict();
    let doc = document::load(&fx.repo, "file.txt").unwrap();
    assert!(doc.change_count >= 1);
    assert_eq!(doc.conflict_count, 1);
    let blue = doc.hunks.iter().find(|h| h.kind.is_blue()).unwrap();
    assert!(blue.left_lines.iter().any(|l| l.contains("BLUE")));
    let conflict = doc.hunks.iter().find(|h| h.kind.is_conflict()).unwrap();
    assert!(conflict.left_lines.iter().any(|l| l.contains("MAIN")));
    assert!(conflict.right_lines.iter().any(|l| l.contains("FEATURE")));
}


#[test]
fn rejects_absolute_path() {
    let fx = support::modify_modify_conflict();
    let result = document::load(&fx.repo, "/etc/passwd");
    assert!(result.is_err());
}

#[test]
fn rejects_path_with_parent_dir_component() {
    let fx = support::modify_modify_conflict();
    let result = document::load(&fx.repo, "../outside.txt");
    assert!(result.is_err());
}
