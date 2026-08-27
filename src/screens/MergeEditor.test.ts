import { describe, it, expect } from "vitest";
import { showAccept } from "./MergeEditor";
import type { HunkKind } from "../types";

const kinds: HunkKind[] = [
  "unchanged",
  "left_change",
  "right_change",
  "both_same",
  "conflict",
];

describe("showAccept", () => {
  it("hides Accept on the unchanged side of a one-sided blue hunk", () => {
    expect(showAccept("left", "left_change")).toBe(true);
    expect(showAccept("right", "left_change")).toBe(false);
    expect(showAccept("left", "right_change")).toBe(false);
    expect(showAccept("right", "right_change")).toBe(true);
  });

  it("keeps both Accepts for both_same and conflict", () => {
    expect(showAccept("left", "both_same")).toBe(true);
    expect(showAccept("right", "both_same")).toBe(true);
    expect(showAccept("left", "conflict")).toBe(true);
    expect(showAccept("right", "conflict")).toBe(true);
  });

  it("does not show Accept on unchanged hunks", () => {
    expect(showAccept("left", "unchanged")).toBe(false);
    expect(showAccept("right", "unchanged")).toBe(false);
  });

  it("covers every hunk kind", () => {
    for (const kind of kinds) {
      expect(typeof showAccept("left", kind)).toBe("boolean");
      expect(typeof showAccept("right", kind)).toBe("boolean");
    }
  });
});
