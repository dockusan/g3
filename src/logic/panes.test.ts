import { describe, it, expect } from "vitest";
import {
  initDecisions,
  countRemaining,
  serializeResult,
  applyDecision,
  type Decision,
} from "./panes";
import type { Region } from "../types";

const regions: Region[] = [
  { kind: "merged", lines: ["line1"] },
  {
    kind: "conflict",
    id: 0,
    ours: ["MAIN"],
    theirs: ["FEATURE"],
    base: ["line2"],
    ours_line_ops: [],
    theirs_line_ops: [],
  },
  { kind: "merged", lines: ["line3"] },
];

describe("pane logic", () => {
  it("starts with all conflicts unresolved", () => {
    const d = initDecisions(regions);
    expect(countRemaining(d)).toBe(1);
  });

  it("accept-ours resolves the conflict", () => {
    let d = initDecisions(regions);
    d = applyDecision(d, 0, "accepted_ours");
    expect(countRemaining(d)).toBe(0);
  });

  it("serializes merged + accepted_ours to the resolved text", () => {
    let d = initDecisions(regions);
    d = applyDecision(d, 0, "accepted_ours");
    expect(serializeResult(regions, d)).toBe("line1\nMAIN\nline3\n");
  });

  it("serializes accepted_theirs", () => {
    let d = initDecisions(regions);
    d = applyDecision(d, 0, "accepted_theirs");
    expect(serializeResult(regions, d)).toBe("line1\nFEATURE\nline3\n");
  });

  it("serializes accepted_both as ours-then-theirs", () => {
    let d = initDecisions(regions);
    d = applyDecision(d, 0, "accepted_both");
    expect(serializeResult(regions, d)).toBe("line1\nMAIN\nFEATURE\nline3\n");
  });

  it("manual override text is used verbatim", () => {
    let d = initDecisions(regions);
    d = applyDecision(d, 0, "manual", ["HANDWRITTEN"]);
    expect(serializeResult(regions, d)).toBe("line1\nHANDWRITTEN\nline3\n");
  });

  it("unresolved conflict serializes back to markers", () => {
    const d = initDecisions(regions);
    const out = serializeResult(regions, d);
    expect(out).toContain("<<<<<<<");
    expect(out).toContain("=======");
    expect(out).toContain(">>>>>>>");
  });
});
