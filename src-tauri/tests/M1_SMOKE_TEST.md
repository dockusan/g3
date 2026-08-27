# Merge Editor Smoke Test (stage-based / IntelliJ chrome)

1. Build a blue+red conflicted repo:
   ```bash
   cd /tmp && rm -rf mt-demo && mkdir mt-demo && cd mt-demo
   git init
   printf 'line1\nline2\nline3\n' > file.txt
   git add . && git commit -m base
   git checkout -b feature
   printf 'line1\nFEATURE\nline3\n' > file.txt
   git commit -am feature
   git checkout master 2>/dev/null || git checkout main
   printf 'line1\nBLUE\nMAIN\nline3\n' > file.txt
   git commit -am main
   git merge feature   # conflict on line2; BLUE is non-conflicting left change
   ```
2. Run `./bin/g3 /tmp/mt-demo`.
3. Overview lists `file.txt` Modified/Modified. Double-click it.
4. Editor shows:
   - Titles `Changes from <ours>` / `Result` / `Changes from <theirs>`
   - Result is **base** (`line1 / line2 / line3`) — no conflict markers
   - Status like `1 change. 1 conflict.`
   - Blue highlight on BLUE hunk; red on MAIN vs FEATURE
   - Apply disabled
5. Click toolbar **» All** → BLUE appears in Result; conflict still pending; Apply still disabled.
6. Click gutter **»** on the conflict (left) or footer **Accept Left** → Result has MAIN; status `0 conflicts`; Apply enabled.
7. **Apply** → overview empty; `cat file.txt` matches decisions; `git status` shows staged, not unmerged.

## Known limitations
- Result is read-only (actions only).
- Whitespace / Highlight dropdowns are stubs.
- No word-level highlighting.
