import { describe, it, expect } from "vitest";
import {
  initDecisions,
  applyDecision,
  applyNonConflicting,
  acceptAllConflicts,
  countPendingChanges,
  countPendingConflicts,
  serializeResult,
} from "./hunks";
import type { Hunk } from "../types";

const hunks: Hunk[] = [
  {
    id: null,
    kind: "unchanged",
    base_lines: ["line1"],
    left_lines: ["line1"],
    right_lines: ["line1"],
    left_line_ops: [],
    right_line_ops: [],
  },
  {
    id: 0,
    kind: "left_change",
    base_lines: [],
    left_lines: ["BLUE"],
    right_lines: [],
    left_line_ops: [],
    right_line_ops: [],
  },
  {
    id: 1,
    kind: "conflict",
    base_lines: ["line2"],
    left_lines: ["MAIN"],
    right_lines: ["FEATURE"],
    left_line_ops: [],
    right_line_ops: [],
  },
  {
    id: null,
    kind: "unchanged",
    base_lines: ["line3"],
    left_lines: ["line3"],
    right_lines: ["line3"],
    left_line_ops: [],
    right_line_ops: [],
  },
];

describe("hunk decisions", () => {
  it("starts with actionable hunks pending", () => {
    const d = initDecisions(hunks);
    expect(d[0]).toBe("pending");
    expect(d[1]).toBe("pending");
    expect(countPendingChanges(hunks, d)).toBe(1);
    expect(countPendingConflicts(hunks, d)).toBe(1);
  });

  it("serializeResult with all pending is pure base", () => {
    const d = initDecisions(hunks);
    expect(serializeResult(hunks, d, true)).toBe("line1\nline2\nline3\n");
  });

  it("applyNonConflicting all accepts blue left lines", () => {
    let d = initDecisions(hunks);
    d = applyNonConflicting(d, hunks, "all");
    expect(d[0]).toBe("accepted_left");
    expect(d[1]).toBe("pending");
    expect(serializeResult(hunks, d, true)).toBe("line1\nBLUE\nline2\nline3\n");
    expect(countPendingConflicts(hunks, d)).toBe(1);
  });

  it("acceptAllConflicts left resolves red only", () => {
    let d = initDecisions(hunks);
    d = acceptAllConflicts(d, hunks, "left");
    expect(d[1]).toBe("accepted_left");
    expect(d[0]).toBe("pending");
    expect(serializeResult(hunks, d, true)).toBe("line1\nMAIN\nline3\n");
    expect(countPendingConflicts(hunks, d)).toBe(0);
  });

  it("keep_base discards a conflict", () => {
    let d = initDecisions(hunks);
    d = applyDecision(d, 1, "keep_base");
    expect(countPendingConflicts(hunks, d)).toBe(0);
    expect(serializeResult(hunks, d, true)).toBe("line1\nline2\nline3\n");
  });

  it("pending blue does not block apply gating helper", () => {
    let d = initDecisions(hunks);
    d = applyDecision(d, 1, "accepted_left");
    expect(countPendingConflicts(hunks, d)).toBe(0);
    expect(countPendingChanges(hunks, d)).toBe(1);
  });
});
