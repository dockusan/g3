import type { ConflictFile } from "../types";

interface Props {
  conflicts: ConflictFile[];
  onOpen: (path: string) => void;
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
          <tr key={c.path} onDoubleClick={() => onOpen(c.path)}>
            <td>{c.path}</td>
            <td>{c.ours_status}</td>
            <td>{c.theirs_status}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
