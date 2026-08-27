use crate::diff::line_diff;
use crate::model::{Hunk, HunkKind, LineOp};
use similar::{capture_diff_slices, Algorithm, DiffOp, DiffTag};

/// Split text into lines **without** keeping the trailing empty line from a
/// final `\n`. Callers pass already-split vectors from `split_lines`.
pub fn build_hunks(base: &[String], ours: &[String], theirs: &[String]) -> Vec<Hunk> {
    build_hunks_lockstep(base, ours, theirs)
}

/// Split file content into lines; return `(lines, trailing_newline)`.
pub fn split_lines(text: &str) -> (Vec<String>, bool) {
    let trailing_newline = text.ends_with('\n');
    let body = if trailing_newline {
        &text[..text.len().saturating_sub(1)]
    } else {
        text
    };
    if body.is_empty() {
        return (Vec::new(), trailing_newline);
    }
    (body.split('\n').map(str::to_string).collect(), trailing_newline)
}

#[derive(Clone, Copy)]
struct Change {
    base_start: usize,
    base_end: usize,
}

struct Span {
    base_start: usize,
    base_end: usize,
    left: bool,
    right: bool,
}

fn build_hunks_lockstep(base: &[String], ours: &[String], theirs: &[String]) -> Vec<Hunk> {
    let left_ops = capture_diff_slices(Algorithm::Myers, base, ours);
    let right_ops = capture_diff_slices(Algorithm::Myers, base, theirs);
    let left_changes = collect_changes(&left_ops);
    let right_changes = collect_changes(&right_ops);
    let spans = merge_spans(&left_changes, &right_changes);

    let mut hunks = Vec::new();
    let mut next_id = 0u32;
    let mut pos = 0usize;
    let mut span_idx = 0usize;

    while pos <= base.len() {
        if span_idx < spans.len() && spans[span_idx].base_start == pos {
            let span = &spans[span_idx];
            span_idx += 1;
            let base_lines = base[span.base_start..span.base_end].to_vec();
            let left_lines = if span.left {
                extract_side(ours, &left_ops, span.base_start, span.base_end)
            } else {
                base_lines.clone()
            };
            let right_lines = if span.right {
                extract_side(theirs, &right_ops, span.base_start, span.base_end)
            } else {
                base_lines.clone()
            };
            let kind = classify(span.left, span.right, &left_lines, &right_lines);
            emit(
                &mut hunks,
                &mut next_id,
                kind,
                base_lines,
                left_lines,
                right_lines,
            );
            if span.base_end > pos {
                pos = span.base_end;
            }
            continue;
        }

        let next_start = spans
            .get(span_idx)
            .map(|s| s.base_start)
            .unwrap_or(base.len());
        if pos < next_start {
            let base_lines = base[pos..next_start].to_vec();
            emit(
                &mut hunks,
                &mut next_id,
                HunkKind::Unchanged,
                base_lines.clone(),
                base_lines.clone(),
                base_lines,
            );
            pos = next_start;
            continue;
        }

        break;
    }

    hunks
}

fn collect_changes(ops: &[DiffOp]) -> Vec<Change> {
    ops.iter()
        .filter(|op| op.tag() != DiffTag::Equal)
        .map(|op| {
            let old = op.old_range();
            Change {
                base_start: old.start,
                base_end: old.end,
            }
        })
        .collect()
}

fn ranges_overlap(a0: usize, a1: usize, b0: usize, b1: usize) -> bool {
    if a0 == a1 && b0 == b1 {
        return a0 == b0;
    }
    a0 < b1 && b0 < a1
}

fn merge_spans(left: &[Change], right: &[Change]) -> Vec<Span> {
    let mut left_used = vec![false; left.len()];
    let mut right_used = vec![false; right.len()];
    let mut spans = Vec::new();

    loop {
        let mut seed: Option<(bool, usize)> = None;
        let mut best = usize::MAX;
        for (i, c) in left.iter().enumerate() {
            if !left_used[i] && c.base_start < best {
                best = c.base_start;
                seed = Some((true, i));
            }
        }
        for (i, c) in right.iter().enumerate() {
            if !right_used[i] && c.base_start < best {
                best = c.base_start;
                seed = Some((false, i));
            }
        }
        let Some((is_left, idx)) = seed else { break };

        let mut start;
        let mut end;
        let mut has_l;
        let mut has_r;
        if is_left {
            left_used[idx] = true;
            start = left[idx].base_start;
            end = left[idx].base_end;
            has_l = true;
            has_r = false;
        } else {
            right_used[idx] = true;
            start = right[idx].base_start;
            end = right[idx].base_end;
            has_l = false;
            has_r = true;
        }

        let mut expanded = true;
        while expanded {
            expanded = false;
            for (i, c) in left.iter().enumerate() {
                if !left_used[i] && ranges_overlap(start, end, c.base_start, c.base_end) {
                    left_used[i] = true;
                    start = start.min(c.base_start);
                    end = end.max(c.base_end);
                    has_l = true;
                    expanded = true;
                }
            }
            for (i, c) in right.iter().enumerate() {
                if !right_used[i] && ranges_overlap(start, end, c.base_start, c.base_end) {
                    right_used[i] = true;
                    start = start.min(c.base_start);
                    end = end.max(c.base_end);
                    has_r = true;
                    expanded = true;
                }
            }
        }

        spans.push(Span {
            base_start: start,
            base_end: end,
            left: has_l,
            right: has_r,
        });
    }

    spans.sort_by_key(|s| (s.base_start, s.base_end));
    spans
}

fn extract_side(side: &[String], ops: &[DiffOp], start: usize, end: usize) -> Vec<String> {
    let mut out = Vec::new();
    for op in ops {
        let old = op.old_range();
        let new = op.new_range();
        match op.tag() {
            DiffTag::Insert => {
                let at = old.start;
                let inside = if start == end {
                    at == start
                } else {
                    at >= start && at < end
                };
                if inside {
                    out.extend(side[new.start..new.end].iter().cloned());
                }
            }
            DiffTag::Equal => {
                let lo = old.start.max(start);
                let hi = old.end.min(end);
                if lo < hi {
                    let new_lo = new.start + (lo - old.start);
                    let new_hi = new_lo + (hi - lo);
                    out.extend(side[new_lo..new_hi].iter().cloned());
                }
            }
            DiffTag::Delete => {}
            DiffTag::Replace => {
                if old.start < end && old.end > start {
                    out.extend(side[new.start..new.end].iter().cloned());
                }
            }
        }
    }
    out
}

fn classify(has_left: bool, has_right: bool, left_lines: &[String], right_lines: &[String]) -> HunkKind {
    match (has_left, has_right) {
        (true, false) => HunkKind::LeftChange,
        (false, true) => HunkKind::RightChange,
        (true, true) if left_lines == right_lines => HunkKind::BothSame,
        (true, true) => HunkKind::Conflict,
        (false, false) => HunkKind::Unchanged,
    }
}

fn lcs_len(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 || n == 0 {
        return 0;
    }
    let mut prev = vec![0usize; n + 1];
    let mut cur = vec![0usize; n + 1];
    for i in 1..=m {
        for j in 1..=n {
            cur[j] = if a[i - 1] == b[j - 1] {
                prev[j - 1] + 1
            } else {
                prev[j].max(cur[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut cur);
        cur.fill(0);
    }
    prev[n]
}

fn lcs_ratio(a: &str, b: &str) -> f64 {
    let denom = a.len().max(b.len());
    if denom == 0 {
        return 1.0;
    }
    lcs_len(a, b) as f64 / denom as f64
}

fn unique_index<T: PartialOrd>(vals: &[T], prefer_max: bool) -> Option<usize> {
    let mut best = 0usize;
    let mut tied = false;
    for i in 1..vals.len() {
        let better = if prefer_max {
            vals[i] > vals[best]
        } else {
            vals[i] < vals[best]
        };
        let equal = vals[i] == vals[best];
        if better {
            best = i;
            tied = false;
        } else if equal {
            tied = true;
        }
    }
    if tied {
        None
    } else {
        Some(best)
    }
}

/// Place extra lines on the longer side as edge inserts when a unique core
/// window of `short.len()` lines can be identified as the 1:1 conflict.
///
/// Exact matches to the other side / base win. Else a window that clearly
/// resembles the other side. Else (all unrelated) the unique *most different*
/// window — extras sit at the edge. Tied windows → no peel (true conflict).
fn unique_core_offset(long: &[String], short: &[String], base: &[String]) -> Option<usize> {
    if short.len() != base.len() || short.is_empty() || long.len() <= short.len() {
        return None;
    }
    let extra = long.len() - short.len();
    let mut exact = Vec::with_capacity(extra + 1);
    let mut weak = Vec::with_capacity(extra + 1);
    for off in 0..=extra {
        let core = &long[off..off + short.len()];
        let mut ex = 0i32;
        let mut w = 0.0f64;
        for i in 0..short.len() {
            if core[i] == short[i] {
                ex += 2;
            } else if core[i] == base[i] {
                ex += 1;
            }
            w += lcs_ratio(&core[i], &short[i]);
        }
        exact.push(ex);
        weak.push(w);
    }
    if let Some(off) = unique_index(&exact, true) {
        if exact[off] > 0 {
            return Some(off);
        }
    }
    if let Some(off) = unique_index(&weak, true) {
        if weak[off] >= 0.5 * short.len() as f64 {
            return Some(off);
        }
    }
    unique_index(&weak, false)
}

fn emit(
    hunks: &mut Vec<Hunk>,
    next_id: &mut u32,
    kind: HunkKind,
    base_lines: Vec<String>,
    left_lines: Vec<String>,
    right_lines: Vec<String>,
) {
    if kind == HunkKind::Conflict && !base_lines.is_empty() {
        if left_lines.len() > base_lines.len() && right_lines.len() == base_lines.len() {
            if let Some(off) = unique_core_offset(&left_lines, &right_lines, &base_lines) {
                let core_len = base_lines.len();
                let prefix = left_lines[..off].to_vec();
                let core = left_lines[off..off + core_len].to_vec();
                let suffix = left_lines[off + core_len..].to_vec();
                if !prefix.is_empty() {
                    emit(
                        hunks,
                        next_id,
                        HunkKind::LeftChange,
                        Vec::new(),
                        prefix,
                        Vec::new(),
                    );
                }
                emit(hunks, next_id, HunkKind::Conflict, base_lines, core, right_lines);
                if !suffix.is_empty() {
                    emit(
                        hunks,
                        next_id,
                        HunkKind::LeftChange,
                        Vec::new(),
                        suffix,
                        Vec::new(),
                    );
                }
                return;
            }
        } else if right_lines.len() > base_lines.len() && left_lines.len() == base_lines.len() {
            if let Some(off) = unique_core_offset(&right_lines, &left_lines, &base_lines) {
                let core_len = base_lines.len();
                let prefix = right_lines[..off].to_vec();
                let core = right_lines[off..off + core_len].to_vec();
                let suffix = right_lines[off + core_len..].to_vec();
                if !prefix.is_empty() {
                    emit(
                        hunks,
                        next_id,
                        HunkKind::RightChange,
                        Vec::new(),
                        Vec::new(),
                        prefix,
                    );
                }
                emit(hunks, next_id, HunkKind::Conflict, base_lines, left_lines, core);
                if !suffix.is_empty() {
                    emit(
                        hunks,
                        next_id,
                        HunkKind::RightChange,
                        Vec::new(),
                        Vec::new(),
                        suffix,
                    );
                }
                return;
            }
        }
    }
    if kind == HunkKind::Unchanged && base_lines.is_empty() {
        return;
    }
    let id = if kind == HunkKind::Unchanged {
        None
    } else {
        let id = *next_id;
        *next_id += 1;
        Some(id)
    };
    let left_line_ops: Vec<LineOp> = line_diff(&base_lines, &left_lines);
    let right_line_ops: Vec<LineOp> = line_diff(&base_lines, &right_lines);
    hunks.push(Hunk {
        id,
        kind,
        base_lines,
        left_lines,
        right_lines,
        left_line_ops,
        right_line_ops,
    });
}



#[cfg(test)]
mod tests {
    use super::*;

    fn s(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|l| l.to_string()).collect()
    }

    fn kinds(hunks: &[Hunk]) -> Vec<HunkKind> {
        hunks.iter().map(|h| h.kind.clone()).collect()
    }

    #[test]
    fn identical_files_single_unchanged() {
        let base = s(&["a", "b"]);
        let hunks = build_hunks(&base, &base, &base);
        assert_eq!(kinds(&hunks), vec![HunkKind::Unchanged]);
        assert_eq!(hunks[0].id, None);
        assert_eq!(hunks[0].base_lines, base);
    }

    #[test]
    fn left_only_edit_is_blue() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "B", "c"]);
        let theirs = base.clone();
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::LeftChange));
        assert!(!hunks.iter().any(|h| h.kind == HunkKind::Conflict));
        let change = hunks.iter().find(|h| h.kind == HunkKind::LeftChange).unwrap();
        assert_eq!(change.left_lines, s(&["B"]));
        assert_eq!(change.base_lines, s(&["b"]));
        assert!(change.id.is_some());
    }

    #[test]
    fn right_only_edit_is_blue() {
        let base = s(&["a", "b", "c"]);
        let ours = base.clone();
        let theirs = s(&["a", "B", "c"]);
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::RightChange));
        assert!(!hunks.iter().any(|h| h.kind == HunkKind::Conflict));
    }

    #[test]
    fn same_edit_both_sides_is_both_same() {
        let base = s(&["a", "b", "c"]);
        let side = s(&["a", "X", "c"]);
        let hunks = build_hunks(&base, &side, &side);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::BothSame));
        assert!(!hunks.iter().any(|h| h.kind == HunkKind::Conflict));
    }

    #[test]
    fn different_edit_same_span_is_conflict() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "L", "c"]);
        let theirs = s(&["a", "R", "c"]);
        let hunks = build_hunks(&base, &ours, &theirs);
        let conflict = hunks.iter().find(|h| h.kind == HunkKind::Conflict).unwrap();
        assert_eq!(conflict.left_lines, s(&["L"]));
        assert_eq!(conflict.right_lines, s(&["R"]));
        assert_eq!(conflict.base_lines, s(&["b"]));
    }

    #[test]
    fn left_only_insert_is_blue() {
        let base = s(&["a", "c"]);
        let ours = s(&["a", "b", "c"]);
        let theirs = base.clone();
        let hunks = build_hunks(&base, &ours, &theirs);
        let ins = hunks.iter().find(|h| h.kind == HunkKind::LeftChange).unwrap();
        assert!(ins.base_lines.is_empty());
        assert_eq!(ins.left_lines, s(&["b"]));
    }

    #[test]
    fn left_only_delete_is_blue() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "c"]);
        let theirs = base.clone();
        let hunks = build_hunks(&base, &ours, &theirs);
        let del = hunks.iter().find(|h| h.kind == HunkKind::LeftChange).unwrap();
        assert_eq!(del.base_lines, s(&["b"]));
        assert!(del.left_lines.is_empty());
    }

    #[test]
    fn delete_vs_modify_is_conflict() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "c"]); // deleted b
        let theirs = s(&["a", "B", "c"]); // modified b
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::Conflict));
    }

    #[test]
    fn empty_base_equal_adds_both_same() {
        let base: Vec<String> = vec![];
        let side = s(&["new"]);
        let hunks = build_hunks(&base, &side, &side);
        assert_eq!(kinds(&hunks), vec![HunkKind::BothSame]);
    }

    #[test]
    fn empty_base_unequal_adds_conflict() {
        let base: Vec<String> = vec![];
        let hunks = build_hunks(&base, &s(&["L"]), &s(&["R"]));
        assert_eq!(kinds(&hunks), vec![HunkKind::Conflict]);
    }

    #[test]
    fn left_insert_adjacent_to_conflict_is_blue_and_red() {
        let base = s(&["line1", "line2", "line3"]);
        let ours = s(&["line1", "BLUE", "MAIN", "line3"]);
        let theirs = s(&["line1", "FEATURE", "line3"]);
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(hunks.iter().any(|h| h.kind.is_blue()));
        assert_eq!(hunks.iter().filter(|h| h.kind.is_conflict()).count(), 1);
        let blue = hunks.iter().find(|h| h.kind.is_blue()).unwrap();
        assert!(blue.left_lines.iter().any(|l| l == "BLUE"));
        let conflict = hunks.iter().find(|h| h.kind.is_conflict()).unwrap();
        assert!(conflict.left_lines.iter().any(|l| l == "MAIN"));
        assert!(conflict.right_lines.iter().any(|l| l == "FEATURE"));
        assert!(!conflict.left_lines.iter().any(|l| l == "BLUE"));
    }

    #[test]
    fn multiline_replace_vs_shorter_stays_conflict() {
        let base = s(&["old"]);
        let ours = s(&["a", "b"]);
        let theirs = s(&["new"]);
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(
            !hunks.iter().any(|h| h.kind.is_blue()),
            "length-only peel must not turn a true multi-line replace into LeftChange"
        );
        assert_eq!(hunks.iter().filter(|h| h.kind.is_conflict()).count(), 1);
        let conflict = hunks.iter().find(|h| h.kind.is_conflict()).unwrap();
        assert_eq!(conflict.base_lines, s(&["old"]));
        assert_eq!(conflict.left_lines, s(&["a", "b"]));
        assert_eq!(conflict.right_lines, s(&["new"]));
    }

    #[test]
    fn suffix_insert_does_not_peel_replacement_as_blue() {
        let base = s(&["line1", "line2", "line3"]);
        let ours = s(&["line1", "MAIN", "BLUE", "line3"]);
        let theirs = s(&["line1", "FEATURE", "line3"]);
        let hunks = build_hunks(&base, &ours, &theirs);
        for h in hunks.iter().filter(|h| h.kind.is_blue()) {
            assert!(
                !h.left_lines.iter().any(|l| l == "MAIN"),
                "prefix peel must not mark MAIN as a blue insert"
            );
        }
        let conflict = hunks.iter().find(|h| h.kind.is_conflict()).unwrap();
        assert!(conflict.left_lines.iter().any(|l| l == "MAIN"));
        assert!(conflict.right_lines.iter().any(|l| l == "FEATURE"));
        let blue = hunks.iter().find(|h| h.kind.is_blue()).expect("BLUE should stay a left insert");
        assert!(blue.left_lines.iter().any(|l| l == "BLUE"));
    }



    #[test]
    fn missing_side_treated_as_empty() {
        // Caller responsibility: pass empty slice. Document that contract.
        let base = s(&["a"]);
        let hunks = build_hunks(&base, &s(&["a"]), &[]);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::RightChange || h.kind == HunkKind::Conflict));
    }

    #[test]
    fn split_lines_trailing_newline() {
        assert_eq!(split_lines("a\nb\n"), (s(&["a", "b"]), true));
        assert_eq!(split_lines("a\nb"), (s(&["a", "b"]), false));
        assert_eq!(split_lines(""), (Vec::<String>::new(), false));
        assert_eq!(split_lines("\n"), (Vec::<String>::new(), true));
    }
}
