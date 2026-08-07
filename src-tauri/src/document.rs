use crate::conflict::{parse_markers, ParsedRegion};
use crate::diff::line_diff;
use crate::git::branch_labels;
use crate::model::{ConflictDocument, Region};
use git2::Repository;

/// Load the on-disk conflicted file, parse marker regions for conflict *locations*,
/// enrich each conflict with diff ops (ours/theirs vs base, or vs each other if no base),
/// and attach branch labels + a content hash for external-change detection.
pub fn load(repo: &Repository, path: &str) -> Result<ConflictDocument, git2::Error> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| git2::Error::from_str("bare repo has no working directory"))?;
    let raw = std::fs::read_to_string(workdir.join(path))
        .map_err(|e| git2::Error::from_str(&format!("read failed: {e}")))?;

    let (ours_label, theirs_label) = branch_labels(repo);

    let parsed = parse_markers(&raw);
    let mut regions = Vec::new();
    let mut id = 0u32;
    let mut total = 0u32;

    for pr in parsed {
        match pr {
            ParsedRegion::Merged { lines } => regions.push(Region::Merged { lines }),
            ParsedRegion::Conflict { ours, theirs, base } => {
                let (ours_line_ops, theirs_line_ops) = match &base {
                    Some(b) => (line_diff(b, &ours), line_diff(b, &theirs)),
                    None => {
                        // No base: diff the two sides against each other for highlighting.
                        (line_diff(&theirs, &ours), line_diff(&ours, &theirs))
                    }
                };
                regions.push(Region::Conflict {
                    id,
                    ours,
                    theirs,
                    base,
                    ours_line_ops,
                    theirs_line_ops,
                });
                id += 1;
                total += 1;
            }
        }
    }

    let content_hash = format!("{:x}", simple_hash(&raw));

    Ok(ConflictDocument {
        path: path.to_string(),
        ours_label,
        theirs_label,
        regions,
        total_conflicts: total,
        content_hash,
    })
}

/// FNV-1a; sufficient for change detection (not security).
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
