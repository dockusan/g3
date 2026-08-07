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

export type Region =
  | { kind: "merged"; lines: string[] }
  | {
      kind: "conflict";
      id: number;
      ours: string[];
      theirs: string[];
      base: string[] | null;
      ours_line_ops: LineOp[];
      theirs_line_ops: LineOp[];
    };

export interface ConflictDocument {
  path: string;
  ours_label: string;
  theirs_label: string;
  regions: Region[];
  total_conflicts: number;
  content_hash: string;
}
