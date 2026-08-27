import type { ConflictFile } from "../types";

export const BINARY_OPEN_MESSAGE = "Cannot open binary conflict in the 3-way editor";

interface Props {
  conflicts: ConflictFile[];
  onOpen: (file: ConflictFile) => void;
}

export function Overview({ conflicts, onOpen }: Props) {
  if (conflicts.length === 0) {
    return <div className="empty">No conflicts. Nothing to resolve.</div>;
  }
  return (
    <table className="overview">
      <thead>
        <tr>
          <th>Name</th>
          <th>Yours</th>
          <th>Theirs</th>
        </tr>
      </thead>
      <tbody>
        {conflicts.map((c) => (
          <tr
            key={c.path}
            title={c.is_binary ? BINARY_OPEN_MESSAGE : undefined}
            onDoubleClick={() => onOpen(c)}
          >
            <td>
              {c.path}
              {c.is_binary ? " (binary)" : ""}
            </td>
            <td>{c.ours_status}</td>
            <td>{c.theirs_status}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
