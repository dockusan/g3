# M1 Smoke Test

This is a manual procedure — it requires a GUI environment and cannot be automated in this repo's current test suite. Run it once after any change that touches the Tauri command layer, the app shell, or the merge editor, to confirm the end-to-end flow still works in a real window.

1. Build a throwaway conflicted repo:
   ```bash
   cd /tmp && rm -rf mt-demo && mkdir mt-demo && cd mt-demo
   git init
   printf 'line1\nline2\nline3\n' > file.txt
   git add . && git commit -m base
   git checkout -b feature
   printf 'line1\nFEATURE\nline3\n' > file.txt
   git commit -am feature
   git checkout master 2>/dev/null || git checkout main
   printf 'line1\nMAIN\nline3\n' > file.txt
   git commit -am main
   git merge feature   # produces a conflict
   ```
2. From `/tmp/mt-demo`, run `g3` (or from this project: `./bin/g3 /tmp/mt-demo`). The window should open against the conflicted repo, not this project's own cwd.
3. Expected: overview lists `file.txt` as Modified / Modified.
4. Double-click it → 3-pane editor shows MAIN (left pane, titled with your current branch name, e.g. "main") and FEATURE (right pane, titled with the merged-in branch name, e.g. "feature") — pane titles come from git branch names, not the literal words "Yours"/"Theirs" (those literal labels only appear as the Overview table's column headers). Result shows conflict markers, "1 conflict(s) remaining", Apply disabled.
5. Click **Accept Left** → Result shows `line1 / MAIN / line3`; header says "No changes. Resolved."; Apply becomes enabled.
6. Click **Apply** → returns to overview, now empty ("No conflicts. Nothing to resolve.").
7. Verify on disk: `cat /tmp/mt-demo/file.txt` → resolved content; `git -C /tmp/mt-demo status` shows `file.txt` staged (resolved), not under "Unmerged paths".

## Known limitations to watch for
- The Result pane in M1 is read-only (live-rendered from Accept Left/Right decisions only) — there's no way to hand-edit individual lines yet. Confirm Accept Left/Right fully resolve the conflict before Apply is expected to be enabled.
- Repo picker flow (standalone mode): launch the built app directly from outside any git repo (or with no conflicts in cwd) — it should show "No conflicts. Nothing to resolve." if in a clean repo, or the "Choose Repository…" picker if not in a repo at all. Use "Choose Repository…" to pick `/tmp/mt-demo` and confirm the same flow works via that entry point too.
