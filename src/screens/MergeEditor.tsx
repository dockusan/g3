import { useMemo, useState } from "react";
import type { ConflictDocument, Region } from "../types";
import {
  initDecisions,
  applyDecision,
  countRemaining,
  serializeResult,
  type Decisions,
} from "../logic/panes";

interface Props {
  doc: ConflictDocument;
  onSave: (content: string) => void;
  onCancel: () => void;
}

function sideLines(regions: Region[], side: "ours" | "theirs"): string[] {
  const out: string[] = [];
  for (const r of regions) {
    if (r.kind === "merged") out.push(...r.lines);
    else out.push(...r[side]);
  }
  return out;
}

export function MergeEditor({ doc, onSave, onCancel }: Props) {
  const [decisions, setDecisions] = useState<Decisions>(() => initDecisions(doc.regions));

  const ours = useMemo(() => sideLines(doc.regions, "ours"), [doc]);
  const theirs = useMemo(() => sideLines(doc.regions, "theirs"), [doc]);
  const result = useMemo(() => serializeResult(doc.regions, decisions), [doc, decisions]);
  const remaining = countRemaining(doc.regions, decisions);

  const conflictIds = doc.regions
    .filter((r): r is Extract<Region, { kind: "conflict" }> => r.kind === "conflict")
    .map((r) => r.id);

  const acceptAll = (kind: "accepted_ours" | "accepted_theirs") => {
    let d = decisions;
    for (const id of conflictIds) d = applyDecision(d, id, kind);
    setDecisions(d);
  };

  return (
    <div className="editor">
      <header>
        <span>{doc.path}</span>
        <span>{remaining === 0 ? "No changes. Resolved." : `${remaining} conflict(s) remaining`}</span>
      </header>
      <div className="panes">
        <pre className="pane ours">
          <div className="pane-title">{doc.ours_label}</div>
          {ours.join("\n")}
        </pre>
        <pre className="pane result">
          <div className="pane-title">Result</div>
          {result}
        </pre>
        <pre className="pane theirs">
          <div className="pane-title">{doc.theirs_label}</div>
          {theirs.join("\n")}
        </pre>
      </div>
      <footer>
        <button onClick={() => acceptAll("accepted_ours")}>Accept Left</button>
        <button onClick={() => acceptAll("accepted_theirs")}>Accept Right</button>
        <span className="spacer" />
        <button onClick={onCancel}>Cancel</button>
        <button disabled={remaining > 0} onClick={() => onSave(result)}>Apply</button>
      </footer>
    </div>
  );
}
