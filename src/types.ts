export type SideStatus = "added" | "modified" | "deleted";

export interface ConflictFile {
  path: string;
  ours_status: SideStatus;
  theirs_status: SideStatus;
  is_binary: boolean;
}

export type LineOp =
  | { op: "equal"; text: string }
  | { op: "insert"; text: string }
  | { op: "delete"; text: string };

export type HunkKind =
  | "unchanged"
  | "left_change"
  | "right_change"
  | "both_same"
  | "conflict";

export interface Hunk {
  id: number | null;
  kind: HunkKind;
  base_lines: string[];
  left_lines: string[];
  right_lines: string[];
  left_line_ops: LineOp[];
  right_line_ops: LineOp[];
}

export interface ConflictDocument {
  path: string;
  ours_label: string;
  theirs_label: string;
  hunks: Hunk[];
  change_count: number;
  conflict_count: number;
  content_hash: string;
  trailing_newline: boolean;
}

export function isBlue(kind: HunkKind): boolean {
  return kind === "left_change" || kind === "right_change" || kind === "both_same";
}

export function isConflict(kind: HunkKind): boolean {
  return kind === "conflict";
}
