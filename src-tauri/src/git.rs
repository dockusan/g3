use crate::model::{ConflictFile, SideStatus};
use git2::Repository;
use std::path::Path;

pub struct Stages {
    pub base: Option<String>,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

pub fn discover_repo(start: &Path) -> Result<Repository, git2::Error> {
    Repository::discover(start)
}

/// Guard against path traversal / absolute paths: reject anything that isn't
/// a plain relative path with no `..` (ParentDir) components. Shared by
/// `writer::save_resolution` and `document::load`, both of which join the
/// caller-supplied path onto the repo's workdir.
pub fn ensure_safe_relative_path(path: &str) -> Result<(), git2::Error> {
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|c| c == std::path::Component::ParentDir)
    {
        return Err(git2::Error::from_str(
            "invalid path: must be relative and contain no '..' components",
        ));
    }
    Ok(())
}

fn blob_to_string(repo: &Repository, oid: git2::Oid) -> Option<String> {
    let blob = repo.find_blob(oid).ok()?;
    Some(String::from_utf8_lossy(blob.content()).to_string())
}

fn is_binary_blob(repo: &Repository, oid: git2::Oid) -> bool {
    match repo.find_blob(oid) {
        Ok(blob) => blob.content().contains(&0u8),
        Err(_) => false,
    }
}

pub fn list_conflicts(repo: &Repository) -> Result<Vec<ConflictFile>, git2::Error> {
    let index = repo.index()?;
    let conflicts = index.conflicts()?;
    let mut out = Vec::new();
    for c in conflicts {
        let c = c?;
        // Determine path from whichever stage exists.
        let path_bytes = c
            .our
            .as_ref()
            .map(|e| e.path.clone())
            .or_else(|| c.their.as_ref().map(|e| e.path.clone()))
            .or_else(|| c.ancestor.as_ref().map(|e| e.path.clone()))
            .unwrap_or_default();
        let path = String::from_utf8_lossy(&path_bytes).to_string();

        let ours_status = side_status(c.ancestor.is_some(), c.our.is_some());
        let theirs_status = side_status(c.ancestor.is_some(), c.their.is_some());

        let our_binary = c.our.as_ref().map_or(false, |e| is_binary_blob(repo, e.id));
        let their_binary = c.their.as_ref().map_or(false, |e| is_binary_blob(repo, e.id));
        let is_binary = our_binary || their_binary;

        out.push(ConflictFile {
            path,
            ours_status,
            theirs_status,
            is_binary,
        });
    }
    Ok(out)
}

fn side_status(has_base: bool, has_side: bool) -> SideStatus {
    match (has_base, has_side) {
        (true, true) => SideStatus::Modified,
        (false, true) => SideStatus::Added,
        (_, false) => SideStatus::Deleted,
    }
}

pub fn read_stages(repo: &Repository, path: &str) -> Result<Stages, git2::Error> {
    let index = repo.index()?;
    let mut base = None;
    let mut ours = None;
    let mut theirs = None;
    for c in index.conflicts()? {
        let c = c?;
        let matches = |e: &Option<git2::IndexEntry>| {
            e.as_ref()
                .map(|x| String::from_utf8_lossy(&x.path) == path)
                .unwrap_or(false)
        };
        if matches(&c.ancestor) || matches(&c.our) || matches(&c.their) {
            if let Some(e) = &c.ancestor {
                base = blob_to_string(repo, e.id);
            }
            if let Some(e) = &c.our {
                ours = blob_to_string(repo, e.id);
            }
            if let Some(e) = &c.their {
                theirs = blob_to_string(repo, e.id);
            }
        }
    }
    Ok(Stages { base, ours, theirs })
}

/// Human-readable labels for the two sides. Ours = current branch (or "Yours");
/// Theirs = the merge head (from MERGE_HEAD ref name if resolvable, else "Theirs").
pub fn branch_labels(repo: &Repository) -> (String, String) {
    let ours = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "Yours".to_string());
    let theirs = std::fs::read_to_string(repo.path().join("MERGE_MSG"))
        .ok()
        .and_then(|msg| {
            // "Merge branch 'origin/main'..." → origin/main
            msg.lines()
                .next()
                .and_then(|l| l.split('\'').nth(1).map(String::from))
        })
        .unwrap_or_else(|| "Theirs".to_string());
    (ours, theirs)
}
