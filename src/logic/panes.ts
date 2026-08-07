import type { Region } from "../types";

export type DecisionKind =
  | "unresolved"
  | "accepted_ours"
  | "accepted_theirs"
  | "accepted_both"
  | "manual";

export interface Decision {
  kind: DecisionKind;
  manualLines?: string[];
}

export type Decisions = Record<number, Decision>;

export function initDecisions(regions: Region[]): Decisions {
  const d: Decisions = {};
  for (const r of regions) {
    if (r.kind === "conflict") d[r.id] = { kind: "unresolved" };
  }
  return d;
}

export function applyDecision(
  decisions: Decisions,
  id: number,
  kind: DecisionKind,
  manualLines?: string[],
): Decisions {
  return { ...decisions, [id]: { kind, manualLines } };
}

export function countRemaining(regions: Region[], decisions: Decisions): number {
  return regions.filter(
    (r) => r.kind === "conflict" && (decisions[r.id]?.kind ?? "unresolved") === "unresolved",
  ).length;
}

/** Resolved text for one conflict region given its decision. */
function conflictText(
  r: Extract<Region, { kind: "conflict" }>,
  d: Decision,
): string[] {
  switch (d.kind) {
    case "accepted_ours":
      return r.ours;
    case "accepted_theirs":
      return r.theirs;
    case "accepted_both":
      return [...r.ours, ...r.theirs];
    case "manual":
      return d.manualLines ?? [];
    case "unresolved":
    default:
      // Re-emit conflict markers so nothing is silently lost.
      return [
        "<<<<<<< ours",
        ...r.ours,
        "=======",
        ...r.theirs,
        ">>>>>>> theirs",
      ];
  }
}

/** Serialize all regions to the final file text (trailing newline included). */
export function serializeResult(regions: Region[], decisions: Decisions): string {
  const out: string[] = [];
  for (const r of regions) {
    if (r.kind === "merged") {
      out.push(...r.lines);
    } else {
      out.push(...conflictText(r, decisions[r.id] ?? { kind: "unresolved" }));
    }
  }
  return out.join("\n") + "\n";
}
