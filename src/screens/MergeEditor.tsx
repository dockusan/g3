import { useMemo, useRef, useState, type UIEvent } from "react";
import type { ConflictDocument, Hunk, HunkKind } from "../types";
import { isBlue, isConflict } from "../types";
import {
  initDecisions,
  applyDecision,
  applyNonConflicting,
  acceptAllConflicts,
  countPendingChanges,
  countPendingConflicts,
  serializeResult,
  type Decisions,
} from "../logic/hunks";

/** Accept belongs on the changed side of a one-sided blue hunk. */
export function showAccept(side: "left" | "right", kind: HunkKind): boolean {
  if (kind === "left_change") return side === "left";
  if (kind === "right_change") return side === "right";
  return kind === "both_same" || kind === "conflict";
}

interface Props {
  doc: ConflictDocument;
  onSave: (content: string) => void;
  onCancel: () => void;
}

function hunkClass(h: Hunk): string {
  if (isConflict(h.kind)) return "hunk conflict";
  if (isBlue(h.kind)) return "hunk change";
  return "hunk";
}

export function MergeEditor({ doc, onSave, onCancel }: Props) {
  const [decisions, setDecisions] = useState<Decisions>(() => initDecisions(doc.hunks));
  const leftRef = useRef<HTMLDivElement>(null);
  const resultRef = useRef<HTMLDivElement>(null);
  const rightRef = useRef<HTMLDivElement>(null);
  const syncing = useRef(false);

  const result = useMemo(
    () => serializeResult(doc.hunks, decisions, doc.trailing_newline),
    [doc, decisions],
  );
  const pendingChanges = countPendingChanges(doc.hunks, decisions);
  const pendingConflicts = countPendingConflicts(doc.hunks, decisions);

  const onScroll = (source: "left" | "result" | "right") => (e: UIEvent<HTMLDivElement>) => {
    if (syncing.current) return;
    syncing.current = true;
    const top = e.currentTarget.scrollTop;
    for (const [name, ref] of [
      ["left", leftRef],
      ["result", resultRef],
      ["right", rightRef],
    ] as const) {
      if (name !== source && ref.current) ref.current.scrollTop = top;
    }
    syncing.current = false;
  };

  return (
    <div className="editor">
      <header className="editor-toolbar">
        <div className="toolbar-group">
          <span>Apply non-conflicting changes:</span>
          <button type="button" onClick={() => setDecisions((d) => applyNonConflicting(d, doc.hunks, "left"))}>
            » Left
          </button>
          <button type="button" onClick={() => setDecisions((d) => applyNonConflicting(d, doc.hunks, "all"))}>
            » All
          </button>
          <button type="button" onClick={() => setDecisions((d) => applyNonConflicting(d, doc.hunks, "right"))}>
            « Right
          </button>
        </div>
        <div className="toolbar-group">
          <select disabled defaultValue="do-not-ignore" aria-label="Whitespace">
            <option value="do-not-ignore">Do not ignore</option>
          </select>
          <select disabled defaultValue="do-not-highlight" aria-label="Highlight">
            <option value="do-not-highlight">Do not highlight</option>
          </select>
        </div>
        <span className="toolbar-status">
          {pendingChanges} change{pendingChanges === 1 ? "" : "s"}. {pendingConflicts} conflict
          {pendingConflicts === 1 ? "" : "s"}.
        </span>
      </header>

      <div className="panes">
        <div className="pane ours" ref={leftRef} onScroll={onScroll("left")}>
          <div className="pane-title">Changes from {doc.ours_label}</div>
          {doc.hunks.map((h, i) => (
            <div key={`L-${i}`} className={hunkClass(h)}>
              {h.id != null && (
                <div className="gutter">
                  {showAccept("left", h.kind) && (
                    <button type="button" title="Accept left" onClick={() => setDecisions((d) => applyDecision(d, h.id!, "accepted_left"))}>
                      »
                    </button>
                  )}
                  <button type="button" title="Keep base" onClick={() => setDecisions((d) => applyDecision(d, h.id!, "keep_base"))}>
                    X
                  </button>
                </div>
              )}
              <pre>{(h.left_lines.length ? h.left_lines : [""]).join("\n")}</pre>
            </div>
          ))}
        </div>

        <div className="pane result" ref={resultRef} onScroll={onScroll("result")}>
          <div className="pane-title">Result</div>
          <pre>{result}</pre>
        </div>

        <div className="pane theirs" ref={rightRef} onScroll={onScroll("right")}>
          <div className="pane-title">Changes from {doc.theirs_label}</div>
          {doc.hunks.map((h, i) => (
            <div key={`R-${i}`} className={hunkClass(h)}>
              {h.id != null && (
                <div className="gutter">
                  {showAccept("right", h.kind) && (
                    <button type="button" title="Accept right" onClick={() => setDecisions((d) => applyDecision(d, h.id!, "accepted_right"))}>
                      «
                    </button>
                  )}
                  <button type="button" title="Keep base" onClick={() => setDecisions((d) => applyDecision(d, h.id!, "keep_base"))}>
                    X
                  </button>
                </div>
              )}
              <pre>{(h.right_lines.length ? h.right_lines : [""]).join("\n")}</pre>
            </div>
          ))}
        </div>
      </div>

      <footer>
        <button type="button" onClick={() => setDecisions((d) => acceptAllConflicts(d, doc.hunks, "left"))}>
          Accept Left
        </button>
        <button type="button" onClick={() => setDecisions((d) => acceptAllConflicts(d, doc.hunks, "right"))}>
          Accept Right
        </button>
        <span className="spacer" />
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
        <button type="button" disabled={pendingConflicts > 0} onClick={() => onSave(result)}>
          Apply
        </button>
      </footer>
    </div>
  );
}
