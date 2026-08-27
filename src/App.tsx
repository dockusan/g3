import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { ConflictFile, ConflictDocument } from "./types";
import { listConflicts, setRepo, loadConflict, saveResolution } from "./api";
import { Overview, BINARY_OPEN_MESSAGE } from "./screens/Overview";
import { MergeEditor } from "./screens/MergeEditor";
import "./App.css";

type View =
  | { screen: "loading" }
  | { screen: "picker" }
  | { screen: "overview" }
  | { screen: "editor"; doc: ConflictDocument };

export default function App() {
  const [conflicts, setConflicts] = useState<ConflictFile[]>([]);
  const [view, setView] = useState<View>({ screen: "loading" });
  const [error, setError] = useState<string | null>(null);

  // On launch, try the repo from `g3 [path]` / `--repo`, else cwd.
  useEffect(() => {
    listConflicts()
      .then((c) => { setConflicts(c); setView({ screen: "overview" }); })
      .catch((e) => { setError(String(e)); setView({ screen: "picker" }); });
  }, []);

  const pickRepo = async () => {
    setError(null);
    try {
      const dir = await open({ directory: true });
      if (typeof dir === "string") {
        const c = await setRepo(dir);
        setConflicts(c);
        setView({ screen: "overview" });
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const openFile = async (file: ConflictFile) => {
    setError(null);
    if (file.is_binary) {
      setError(BINARY_OPEN_MESSAGE);
      return;
    }
    try {
      const doc = await loadConflict(file.path);
      setView({ screen: "editor", doc });
    } catch (e) {
      setError(String(e));
    }
  };

  const save = async (content: string) => {
    if (view.screen !== "editor") return;
    setError(null);
    try {
      const c = await saveResolution(view.doc.path, content);
      setConflicts(c);
      setView({ screen: "overview" });
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="app">
      {error && <div className="error" onClick={() => setError(null)}>{error}</div>}
      {view.screen === "loading" && <div className="loading">Loading…</div>}
      {view.screen === "picker" && (
        <div className="picker">
          <p>Open a git repository with merge conflicts.</p>
          <button onClick={pickRepo}>Choose Repository…</button>
        </div>
      )}
      {view.screen === "overview" && (
        <Overview conflicts={conflicts} onOpen={openFile} />
      )}
      {view.screen === "editor" && (
        <MergeEditor
          key={view.doc.path}
          doc={view.doc}
          onSave={save}
          onCancel={() => setView({ screen: "overview" })}
        />
      )}
    </div>
  );
}
