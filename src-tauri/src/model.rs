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
