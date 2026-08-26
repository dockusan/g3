use std::path::{Path, PathBuf};

/// Resolve the git repo to open from `g3` CLI args.
///
/// - `g3` → `cwd`
/// - `g3 /path/to/repo` → that path (relative paths are joined to `cwd`)
/// - `g3 --repo <path>` → that path
pub fn resolve_repo_path(args: &[String], cwd: &Path) -> PathBuf {
    let rest = args.get(1..).unwrap_or(&[]);

    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--repo" {
            if let Some(p) = rest.get(i + 1) {
                return absolutize(Path::new(p), cwd);
            }
            break;
        }
        i += 1;
    }

    for arg in rest {
        if !arg.starts_with('-') {
            return absolutize(Path::new(arg), cwd);
        }
    }

    cwd.to_path_buf()
}

fn absolutize(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
