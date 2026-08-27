import type { Hunk, HunkKind } from "../types";
import { isBlue, isConflict } from "../types";

export type DecisionKind =
  | "pending"
  | "accepted_left"
  | "accepted_right"
  | "keep_base";

export type Decisions = Record<number, DecisionKind>;

export function initDecisions(hunks: Hunk[]): Decisions {
  const d: Decisions = {};
  for (const h of hunks) {
    if (h.id != null) d[h.id] = "pending";
  }
  return d;
}

export function applyDecision(
  decisions: Decisions,
  id: number,
  kind: DecisionKind,
): Decisions {
  return { ...decisions, [id]: kind };
}

function pendingOf(
  hunks: Hunk[],
  decisions: Decisions,
  pred: (k: HunkKind) => boolean,
): Hunk[] {
  return hunks.filter(
    (h) =>
      h.id != null &&
      pred(h.kind) &&
      (decisions[h.id] ?? "pending") === "pending",
  );
}

export function applyNonConflicting(
  decisions: Decisions,
  hunks: Hunk[],
  side: "left" | "right" | "all",
): Decisions {
  let d = decisions;
  for (const h of pendingOf(hunks, d, isBlue)) {
    const id = h.id!;
    if (side === "left") {
      if (h.kind === "left_change" || h.kind === "both_same") {
        d = applyDecision(d, id, "accepted_left");
      }
    } else if (side === "right") {
      if (h.kind === "right_change" || h.kind === "both_same") {
        d = applyDecision(d, id, "accepted_right");
      }
    } else {
      // all: left blues → left, right blues → right, both_same → left
      if (h.kind === "right_change") d = applyDecision(d, id, "accepted_right");
      else d = applyDecision(d, id, "accepted_left");
    }
  }
  return d;
}

export function acceptAllConflicts(
  decisions: Decisions,
  hunks: Hunk[],
  side: "left" | "right",
): Decisions {
  let d = decisions;
  const kind = side === "left" ? "accepted_left" : "accepted_right";
  for (const h of pendingOf(hunks, d, isConflict)) {
    d = applyDecision(d, h.id!, kind);
  }
  return d;
}

export function countPendingChanges(hunks: Hunk[], decisions: Decisions): number {
  return pendingOf(hunks, decisions, isBlue).length;
}

export function countPendingConflicts(hunks: Hunk[], decisions: Decisions): number {
  return pendingOf(hunks, decisions, isConflict).length;
}

function linesFor(h: Hunk, decision: DecisionKind | undefined): string[] {
  switch (decision ?? "pending") {
    case "accepted_left":
      return h.left_lines;
    case "accepted_right":
      return h.right_lines;
    case "keep_base":
    case "pending":
    default:
      return h.base_lines;
  }
}

export function serializeResult(
  hunks: Hunk[],
  decisions: Decisions,
  trailingNewline: boolean,
): string {
  const out: string[] = [];
  for (const h of hunks) {
    const decision = h.id == null ? "pending" : decisions[h.id];
    out.push(...linesFor(h, decision));
  }
  const body = out.join("\n");
  if (body.length === 0) return trailingNewline ? "\n" : "";
  return trailingNewline ? body + "\n" : body;
}
