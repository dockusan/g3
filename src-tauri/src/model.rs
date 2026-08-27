use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SideStatus {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictFile {
    pub path: String,
    pub ours_status: SideStatus,
    pub theirs_status: SideStatus,
    pub is_binary: bool,
}

/// One line-diff operation for a side, vs the base (or vs the other side when no base).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum LineOp {
    Equal { text: String },
    Insert { text: String },
    Delete { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Region {
    Merged {
        lines: Vec<String>,
    },
    Conflict {
        id: u32,
        ours: Vec<String>,
        theirs: Vec<String>,
        base: Option<Vec<String>>,
        ours_line_ops: Vec<LineOp>,
        theirs_line_ops: Vec<LineOp>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictDocument {
    pub path: String,
    pub ours_label: String,
    pub theirs_label: String,
    pub regions: Vec<Region>,
    pub total_conflicts: u32,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HunkKind {
    Unchanged,
    LeftChange,
    RightChange,
    BothSame,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hunk {
    /// Present only for actionable hunks (not `Unchanged`).
    pub id: Option<u32>,
    pub kind: HunkKind,
    pub base_lines: Vec<String>,
    pub left_lines: Vec<String>,
    pub right_lines: Vec<String>,
    pub left_line_ops: Vec<LineOp>,
    pub right_line_ops: Vec<LineOp>,
}

impl HunkKind {
    pub fn is_blue(self) -> bool {
        matches!(
            self,
            HunkKind::LeftChange | HunkKind::RightChange | HunkKind::BothSame
        )
    }

    pub fn is_conflict(self) -> bool {
        matches!(self, HunkKind::Conflict)
    }

    pub fn is_actionable(self) -> bool {
        !matches!(self, HunkKind::Unchanged)
    }
}
