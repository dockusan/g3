use git2::Repository;
use std::path::Path;

/// Write `content` to `<workdir>/<path>` and stage it, clearing the conflict.
pub fn save_resolution(repo: &Repository, path: &str, content: &str) -> Result<(), git2::Error> {
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

    let workdir = repo
        .workdir()
        .ok_or_else(|| git2::Error::from_str("bare repo has no working directory"))?;
    let full = workdir.join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| git2::Error::from_str(&format!("mkdir failed: {e}")))?;
    }
    std::fs::write(&full, content)
        .map_err(|e| git2::Error::from_str(&format!("write failed: {e}")))?;

    let mut index = repo.index()?;
    // add_path stages the working-tree version and removes the conflict stages.
    index.add_path(Path::new(path))?;
    index.write()?;
    Ok(())
}
