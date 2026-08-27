use crate::conflict::{parse_markers, ParsedRegion};
use crate::diff::line_diff;
use crate::git::{branch_labels, ensure_safe_relative_path};
use crate::merge3::{build_hunks, split_lines};
use crate::model::{ConflictDocument, Hunk, HunkKind};
use git2::Repository;

/// Load a conflicted file as stage-based hunks when index stages exist;
/// otherwise fall back to parsing conflict markers in the working tree.
pub fn load(repo: &Repository, path: &str) -> Result<ConflictDocument, git2::Error> {
    ensure_safe_relative_path(path)?;
    let (ours_label, theirs_label) = branch_labels(repo);
    let stages = crate::git::read_stages(repo, path)?;

    // Prefer stages when at least one side blob exists.
    let usable = stages.ours.is_some() || stages.theirs.is_some() || stages.base.is_some();
    if usable {
        let base_text = stages.base.clone().unwrap_or_default();
        let ours_text = stages.ours.clone().unwrap_or_default();
        let theirs_text = stages.theirs.clone().unwrap_or_default();
        let (base, base_nl) = split_lines(&base_text);
        let (ours, ours_nl) = split_lines(&ours_text);
        let (theirs, theirs_nl) = split_lines(&theirs_text);
        // Prefer ours trailing newline if present, else theirs, else base.
        let trailing_newline = ours_nl || theirs_nl || base_nl;
        let hunks = build_hunks(&base, &ours, &theirs);
        let change_count = hunks.iter().filter(|h| h.kind.is_blue()).count() as u32;
        let conflict_count = hunks.iter().filter(|h| h.kind.is_conflict()).count() as u32;
        let content_hash = format!(
            "{:x}",
            simple_hash(&format!(
                "B\0{}\0O\0{}\0T\0{}",
                base_text, ours_text, theirs_text
            ))
        );
        return Ok(ConflictDocument {
            path: path.to_string(),
            ours_label,
            theirs_label,
            hunks,
            change_count,
            conflict_count,
            content_hash,
            trailing_newline,
        });
    }

    // Fallback: marker parse of working tree file.
    let workdir = repo
        .workdir()
        .ok_or_else(|| git2::Error::from_str("bare repo has no working directory"))?;
    let raw = std::fs::read_to_string(workdir.join(path))
        .map_err(|e| git2::Error::from_str(&format!("read failed: {e}")))?;
    let trailing_newline = raw.ends_with('\n');
    let parsed = parse_markers(&raw);
    let mut hunks = Vec::new();
    let mut next_id = 0u32;
    for pr in parsed {
        match pr {
            ParsedRegion::Merged { lines } => hunks.push(Hunk {
                id: None,
                kind: HunkKind::Unchanged,
                base_lines: lines.clone(),
                left_lines: lines.clone(),
                right_lines: lines,
                left_line_ops: vec![],
                right_line_ops: vec![],
            }),
            ParsedRegion::Conflict { ours, theirs, base } => {
                let base_lines = base.clone().unwrap_or_default();
                hunks.push(Hunk {
                    id: Some(next_id),
                    kind: HunkKind::Conflict,
                    left_line_ops: line_diff(&base_lines, &ours),
                    right_line_ops: line_diff(&base_lines, &theirs),
                    base_lines,
                    left_lines: ours,
                    right_lines: theirs,
                });
                next_id += 1;
            }
        }
    }
    let change_count = 0;
    let conflict_count = hunks.iter().filter(|h| h.kind.is_conflict()).count() as u32;
    Ok(ConflictDocument {
        path: path.to_string(),
        ours_label,
        theirs_label,
        hunks,
        change_count,
        conflict_count,
        content_hash: format!("{:x}", simple_hash(&raw)),
        trailing_newline,
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
