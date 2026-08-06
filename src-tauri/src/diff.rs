use crate::model::LineOp;
use similar::{ChangeTag, TextDiff};

/// Line-level diff of `a` (base/left) vs `b` (right), as ops describing how to get
/// from `a` to `b`. Equal lines appear once; changed lines appear as Delete (from a)
/// then Insert (from b).
pub fn line_diff(a: &[String], b: &[String]) -> Vec<LineOp> {
    let a_text = a.join("\n");
    let b_text = b.join("\n");
    let diff = TextDiff::from_lines(&a_text, &b_text);
    let mut ops = Vec::new();
    for change in diff.iter_all_changes() {
        let text = change.value().trim_end_matches('\n').to_string();
        match change.tag() {
            ChangeTag::Equal => ops.push(LineOp::Equal { text }),
            ChangeTag::Delete => ops.push(LineOp::Delete { text }),
            ChangeTag::Insert => ops.push(LineOp::Insert { text }),
        }
    }
    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_input_is_all_equal() {
        let a = vec!["x".to_string(), "y".to_string()];
        let ops = line_diff(&a, &a);
        assert!(ops.iter().all(|o| matches!(o, LineOp::Equal { .. })));
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn added_line_is_insert() {
        let a = vec!["x".to_string()];
        let b = vec!["x".to_string(), "y".to_string()];
        let ops = line_diff(&a, &b);
        assert!(ops.iter().any(|o| matches!(o, LineOp::Insert { text } if text == "y")));
    }

    #[test]
    fn removed_line_is_delete() {
        let a = vec!["x".to_string(), "y".to_string()];
        let b = vec!["x".to_string()];
        let ops = line_diff(&a, &b);
        assert!(ops.iter().any(|o| matches!(o, LineOp::Delete { text } if text == "y")));
    }
}
