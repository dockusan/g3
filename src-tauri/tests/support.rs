// Shared helper for building a temp git repo with a real modify/modify conflict.
// Used by git_test.rs and writer_test.rs via `#[path]` include.
use git2::{Repository, Signature};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

pub struct Fixture {
    pub dir: TempDir,
    pub repo: Repository,
}

fn commit_all(repo: &Repository, msg: &str) -> git2::Oid {
    let mut index = repo.index().unwrap();
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = Signature::now("Test", "test@example.com").unwrap();
    let parents = match repo.head() {
        Ok(head) => vec![head.peel_to_commit().unwrap()],
        Err(_) => vec![],
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs).unwrap()
}

fn write(path: &Path, name: &str, contents: &str) {
    fs::write(path.join(name), contents).unwrap();
}

/// Build a repo where `file.txt` is modified differently on `main` and `feature`,
/// then merge feature into main to produce a conflict in the working tree.
pub fn modify_modify_conflict() -> Fixture {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();

    write(dir.path(), "file.txt", "line1\nline2\nline3\n");
    commit_all(&repo, "base");

    // Capture the actual initial branch name created by `Repository::init`,
    // since it depends on the system/global `init.defaultBranch` config
    // (commonly "main" on modern setups, but "master" elsewhere).
    let default_branch = repo
        .head()
        .unwrap()
        .name()
        .expect("HEAD ref name should be valid UTF-8")
        .to_string();

    // feature branch changes line2
    {
        let base_commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &base_commit, false).unwrap();
    }
    repo.set_head("refs/heads/feature").unwrap();
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
    write(dir.path(), "file.txt", "line1\nFEATURE\nline3\n");
    commit_all(&repo, "feature change");

    // back to the original default branch, change line2 differently
    repo.set_head(&default_branch).unwrap();
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
    write(dir.path(), "file.txt", "line1\nMAIN\nline3\n");
    commit_all(&repo, "main change");

    // merge feature -> produces conflict
    let feature_commit_id = repo
        .find_branch("feature", git2::BranchType::Local).unwrap()
        .get().peel_to_commit().unwrap()
        .id();
    {
        let annotated = repo.find_annotated_commit(feature_commit_id).unwrap();
        let mut opts = git2::MergeOptions::new();
        repo.merge(&[&annotated], Some(&mut opts), None).unwrap();
    }

    Fixture { dir, repo }
}
