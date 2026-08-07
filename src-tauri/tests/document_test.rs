#[path = "support.rs"]
mod support;

use tauri_app_lib::{document, model::Region};

#[test]
fn builds_document_with_one_conflict_region() {
    let fx = support::modify_modify_conflict();
    let doc = document::load(&fx.repo, "file.txt").unwrap();
    assert_eq!(doc.path, "file.txt");
    assert_eq!(doc.total_conflicts, 1);
    assert!(!doc.content_hash.is_empty());

    let conflict_count = doc.regions.iter()
        .filter(|r| matches!(r, Region::Conflict { .. }))
        .count();
    assert_eq!(conflict_count, 1);

    // The conflict carries ours/theirs content and diff ops.
    for r in &doc.regions {
        if let Region::Conflict { ours, theirs, ours_line_ops, theirs_line_ops, .. } = r {
            assert!(ours.iter().any(|l| l.contains("MAIN")));
            assert!(theirs.iter().any(|l| l.contains("FEATURE")));
            assert!(!ours_line_ops.is_empty());
            assert!(!theirs_line_ops.is_empty());
        }
    }
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
