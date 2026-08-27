# IntelliJ-Style Merge Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the M1 marker-based 3-pane editor with a stage-based IntelliJ-style merge UI (blue non-conflicts, red conflicts, pure-base Result, gutters + Apply non-conflicting toolbar).

**Architecture:** `git::read_stages` → `merge3::build_hunks` → `ConflictDocument.hunks`. Frontend decisions (`pending` / `accepted_left` / `accepted_right` / `keep_base`) drive a read-only Result via `serializeResult`. Marker parsing remains fallback only.

**Tech Stack:** Rust (Tauri 2, git2, similar), React 19 + TypeScript, Vitest, Cargo tests.

**Spec:** `docs/superpowers/specs/2026-08-27-intellij-merge-editor-design.md`

## Global Constraints

- Result opens as **pure base**; blue hunks stay pending until applied.
- Result is **actions-only** (no typing).
- Gutter `X` = `keep_base` (mark resolved).
- Highlighting is **line/block only** (no word-level).
- Toolbar dropdowns are **stubs** (`Do not ignore`, `Do not highlight` only).
- Apply enabled iff **pending conflicts === 0**; pending blues do **not** block Apply (they stay as base).
- Keep IPC names `load_conflict` / `save_resolution`.
- No Accept Both / manual override this pass.
- Prefer `Option<u32>` ids on hunks (only actionable hunks get ids).
- Always preserve trailing-newline semantics via `ConflictDocument.trailing_newline`.

---

## File map

| File | Responsibility |
|---|---|
| `src-tauri/src/model.rs` | `HunkKind`, `Hunk`, updated `ConflictDocument`; retire editor-facing `Region` |
| `src-tauri/src/merge3.rs` | Pure 3-way hunk builder + unit tests |
| `src-tauri/src/document.rs` | Load stages → hunks; marker fallback |
| `src-tauri/src/lib.rs` | `pub mod merge3` |
| `src-tauri/tests/support.rs` | Add blue+red fixture |
| `src-tauri/tests/document_test.rs` | Assert hunk-based load |
| `src/types.ts` | Mirror hunk document types |
| `src/logic/hunks.ts` | Decisions + serialize + counts |
| `src/logic/hunks.test.ts` | Vitest coverage |
| `src/screens/MergeEditor.tsx` | IntelliJ chrome UI |
| `src/App.css` | Toolbar / panes / gutters / colors |
| `src-tauri/tests/M1_SMOKE_TEST.md` | Updated manual procedure |
| Delete after cutover | `src/logic/panes.ts`, `src/logic/panes.test.ts` |

---

### Task 1: Model types — `Hunk` + new `ConflictDocument`

**Files:**
- Modify: `src-tauri/src/model.rs`
- Modify: any compile breakers temporarily (keep old `Region` until Task 3 if needed — prefer replace in one commit with Task 3; this task adds new types alongside `Region`, then Task 3 removes `Region`)

**Interfaces:**
- Produces:
  - `HunkKind::{Unchanged, LeftChange, RightChange, BothSame, Conflict}`
  - `Hunk { id: Option<u32>, kind, base_lines, left_lines, right_lines, left_line_ops, right_line_ops }`
  - `ConflictDocument { path, ours_label, theirs_label, hunks, change_count, conflict_count, content_hash, trailing_newline }`

- [ ] **Step 1: Add new types alongside existing `Region`**

In `src-tauri/src/model.rs`, keep `Region` for now and append:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HunkKind {
    Unchanged,
    LeftChange,
    RightChange,
    BothSame,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hunk {
    /// Present only for actionable hunks (not `Unchanged`).
    pub id: Option<u32>,
    pub kind: HunkKind,
    pub base_lines: Vec<String>,
    pub left_lines: Vec<String>,
    pub right_lines: Vec<String>,
    pub left_line_ops: Vec<LineOp>,
    pub right_line_ops: Vec<LineOp>,
}

impl HunkKind {
    pub fn is_blue(self) -> bool {
        matches!(
            self,
            HunkKind::LeftChange | HunkKind::RightChange | HunkKind::BothSame
        )
    }

    pub fn is_conflict(self) -> bool {
        matches!(self, HunkKind::Conflict)
    }

    pub fn is_actionable(self) -> bool {
        !matches!(self, HunkKind::Unchanged)
    }
}
```

Also add a **new** document shape used by later tasks (do not rename `ConflictDocument` yet — add helper constructors later in Task 3). For this step only add `Hunk` / `HunkKind` so merge3 can compile.

- [ ] **Step 2: Compile-check**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/model.rs
git commit -m "feat: add Hunk and HunkKind model types"
```

---

### Task 2: `merge3::build_hunks` (TDD)

**Files:**
- Create: `src-tauri/src/merge3.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod merge3;`)
- Test: unit tests inside `merge3.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `Hunk`, `HunkKind`, `LineOp`, `diff::line_diff`
- Produces: `pub fn build_hunks(base: &[String], ours: &[String], theirs: &[String]) -> Vec<Hunk>`

- [ ] **Step 1: Wire empty module + failing test**

`lib.rs` — add `pub mod merge3;` after `pub mod diff;`.

Create `merge3.rs`:

```rust
use crate::diff::line_diff;
use crate::model::{Hunk, HunkKind, LineOp};

/// Split text into lines **without** keeping the trailing empty line from a
/// final `\n`. Callers pass already-split vectors from `split_lines`.
pub fn build_hunks(base: &[String], ours: &[String], theirs: &[String]) -> Vec<Hunk> {
    todo!("build_hunks")
}

/// Split file content into lines; return `(lines, trailing_newline)`.
pub fn split_lines(text: &str) -> (Vec<String>, bool) {
    let trailing_newline = text.ends_with('\n');
    let body = if trailing_newline {
        &text[..text.len().saturating_sub(1)]
    } else {
        text
    };
    if body.is_empty() {
        return (Vec::new(), trailing_newline);
    }
    (body.split('\n').map(str::to_string).collect(), trailing_newline)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|l| l.to_string()).collect()
    }

    fn kinds(hunks: &[Hunk]) -> Vec<HunkKind> {
        hunks.iter().map(|h| h.kind.clone()).collect()
    }

    #[test]
    fn identical_files_single_unchanged() {
        let base = s(&["a", "b"]);
        let hunks = build_hunks(&base, &base, &base);
        assert_eq!(kinds(&hunks), vec![HunkKind::Unchanged]);
        assert_eq!(hunks[0].id, None);
        assert_eq!(hunks[0].base_lines, base);
    }
}
```

- [ ] **Step 2: Run test — expect fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml identical_files_single_unchanged -- --nocapture`

Expected: FAIL (`todo!` panic or link error)

- [ ] **Step 3: Implement `build_hunks`**

Use a base-aligned zip of left/right diffs. Concrete approach:

1. Build `TextDiff` base→ours and base→theirs with `similar::TextDiff::from_slices` on `&[String]` slices (or join/`from_lines` consistently).
2. Walk both diffs by consuming **base lines** in lockstep:
   - Collect runs of Equal on both sides → `Unchanged`.
   - Where only left changes (delete/insert vs base) and right stays Equal for those base lines → `LeftChange`.
   - Symmetric → `RightChange`.
   - Same replacement on both sides (`left_lines == right_lines`, both ≠ base) → `BothSame`.
   - Different replacements overlapping the same base span → `Conflict`.
3. Pure inserts (no base lines): if only left inserts → `LeftChange` with empty `base_lines`; only right → `RightChange`; both insert same → `BothSame`; both insert different → `Conflict`.
4. Assign `id = Some(n)` monotonically for every actionable hunk; `Unchanged` gets `id = None`.
5. Set `left_line_ops = line_diff(&base_lines, &left_lines)` and `right_line_ops = line_diff(&base_lines, &right_lines)` on each hunk.

Reference skeleton (implement fully — do not leave `todo!`):

```rust
pub fn build_hunks(base: &[String], ours: &[String], theirs: &[String]) -> Vec<Hunk> {
    // Preferred implementation path if a compact lockstep walker is hard:
    // fall back to `similar::TextDiff` ops + a small state machine that emits
    // spans. Keep the public behavior identical to the tests below.
    build_hunks_lockstep(base, ours, theirs)
}
```

If the lockstep walker is large, keep helpers private in the same file (`emit`, `classify_span`, etc.). Do **not** invent a second public API.

- [ ] **Step 4: Add the full edge-case tests and make them pass**

Append tests (all must pass):

```rust
    #[test]
    fn left_only_edit_is_blue() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "B", "c"]);
        let theirs = base.clone();
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::LeftChange));
        assert!(!hunks.iter().any(|h| h.kind == HunkKind::Conflict));
        let change = hunks.iter().find(|h| h.kind == HunkKind::LeftChange).unwrap();
        assert_eq!(change.left_lines, s(&["B"]));
        assert_eq!(change.base_lines, s(&["b"]));
        assert!(change.id.is_some());
    }

    #[test]
    fn right_only_edit_is_blue() {
        let base = s(&["a", "b", "c"]);
        let ours = base.clone();
        let theirs = s(&["a", "B", "c"]);
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::RightChange));
        assert!(!hunks.iter().any(|h| h.kind == HunkKind::Conflict));
    }

    #[test]
    fn same_edit_both_sides_is_both_same() {
        let base = s(&["a", "b", "c"]);
        let side = s(&["a", "X", "c"]);
        let hunks = build_hunks(&base, &side, &side);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::BothSame));
        assert!(!hunks.iter().any(|h| h.kind == HunkKind::Conflict));
    }

    #[test]
    fn different_edit_same_span_is_conflict() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "L", "c"]);
        let theirs = s(&["a", "R", "c"]);
        let hunks = build_hunks(&base, &ours, &theirs);
        let conflict = hunks.iter().find(|h| h.kind == HunkKind::Conflict).unwrap();
        assert_eq!(conflict.left_lines, s(&["L"]));
        assert_eq!(conflict.right_lines, s(&["R"]));
        assert_eq!(conflict.base_lines, s(&["b"]));
    }

    #[test]
    fn left_only_insert_is_blue() {
        let base = s(&["a", "c"]);
        let ours = s(&["a", "b", "c"]);
        let theirs = base.clone();
        let hunks = build_hunks(&base, &ours, &theirs);
        let ins = hunks.iter().find(|h| h.kind == HunkKind::LeftChange).unwrap();
        assert!(ins.base_lines.is_empty());
        assert_eq!(ins.left_lines, s(&["b"]));
    }

    #[test]
    fn left_only_delete_is_blue() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "c"]);
        let theirs = base.clone();
        let hunks = build_hunks(&base, &ours, &theirs);
        let del = hunks.iter().find(|h| h.kind == HunkKind::LeftChange).unwrap();
        assert_eq!(del.base_lines, s(&["b"]));
        assert!(del.left_lines.is_empty());
    }

    #[test]
    fn delete_vs_modify_is_conflict() {
        let base = s(&["a", "b", "c"]);
        let ours = s(&["a", "c"]); // deleted b
        let theirs = s(&["a", "B", "c"]); // modified b
        let hunks = build_hunks(&base, &ours, &theirs);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::Conflict));
    }

    #[test]
    fn empty_base_equal_adds_both_same() {
        let base: Vec<String> = vec![];
        let side = s(&["new"]);
        let hunks = build_hunks(&base, &side, &side);
        assert_eq!(kinds(&hunks), vec![HunkKind::BothSame]);
    }

    #[test]
    fn empty_base_unequal_adds_conflict() {
        let base: Vec<String> = vec![];
        let hunks = build_hunks(&base, &s(&["L"]), &s(&["R"]));
        assert_eq!(kinds(&hunks), vec![HunkKind::Conflict]);
    }

    #[test]
    fn missing_side_treated_as_empty() {
        // Caller responsibility: pass empty slice. Document that contract.
        let base = s(&["a"]);
        let hunks = build_hunks(&base, &s(&["a"]), &[]);
        assert!(hunks.iter().any(|h| h.kind == HunkKind::RightChange || h.kind == HunkKind::Conflict));
    }

    #[test]
    fn split_lines_trailing_newline() {
        assert_eq!(split_lines("a\nb\n"), (s(&["a", "b"]), true));
        assert_eq!(split_lines("a\nb"), (s(&["a", "b"]), false));
        assert_eq!(split_lines(""), (Vec::<String>::new(), false));
        assert_eq!(split_lines("\n"), (Vec::<String>::new(), true));
    }
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml merge3 -- --nocapture`

Expected: all PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/merge3.rs src-tauri/src/lib.rs
git commit -m "feat: add merge3 hunk builder with edge-case tests"
```

---

### Task 3: Switch `document::load` to stages → hunks

**Files:**
- Modify: `src-tauri/src/model.rs` (replace `ConflictDocument.regions` / `total_conflicts` with hunk fields; remove `Region` if unused)
- Modify: `src-tauri/src/document.rs`
- Modify: `src-tauri/tests/document_test.rs`
- Modify: `src-tauri/tests/support.rs` (add blue+red fixture)
- Grep for `Region` / `total_conflicts` / `.regions` and update all Rust call sites

**Interfaces:**
- Consumes: `git::read_stages`, `merge3::{build_hunks, split_lines}`, `conflict::parse_markers`
- Produces: `ConflictDocument` with `hunks`, `change_count`, `conflict_count`, `trailing_newline`

- [ ] **Step 1: Rewrite `ConflictDocument` and remove `Region`**

Replace `Region` + old document fields with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictDocument {
    pub path: String,
    pub ours_label: String,
    pub theirs_label: String,
    pub hunks: Vec<Hunk>,
    pub change_count: u32,
    pub conflict_count: u32,
    pub content_hash: String,
    pub trailing_newline: bool,
}
```

Delete the `Region` enum from `model.rs` once nothing references it.

- [ ] **Step 2: Rewrite `document::load`**

```rust
pub fn load(repo: &Repository, path: &str) -> Result<ConflictDocument, git2::Error> {
    ensure_safe_relative_path(path)?;
    let (ours_label, theirs_label) = branch_labels(repo);
    let stages = crate::git::read_stages(repo, path)?;

    // Prefer stages when at least one side blob exists.
    let usable = stages.ours.is_some() || stages.theirs.is_some() || stages.base.is_some();
    if usable {
        let base_text = stages.base.clone().unwrap_or_default();
        let ours_text = stages.ours.clone().unwrap_or_default();
        let theirs_text = stages.theirs.clone().unwrap_or_default();
        let (base, base_nl) = split_lines(&base_text);
        let (ours, ours_nl) = split_lines(&ours_text);
        let (theirs, theirs_nl) = split_lines(&theirs_text);
        // Prefer ours trailing newline if present, else theirs, else base.
        let trailing_newline = ours_nl || theirs_nl || base_nl;
        let hunks = build_hunks(&base, &ours, &theirs);
        let change_count = hunks.iter().filter(|h| h.kind.is_blue()).count() as u32;
        let conflict_count = hunks.iter().filter(|h| h.kind.is_conflict()).count() as u32;
        let content_hash = format!(
            "{:x}",
            simple_hash(&format!(
                "B\0{}\0O\0{}\0T\0{}",
                base_text, ours_text, theirs_text
            ))
        );
        return Ok(ConflictDocument {
            path: path.to_string(),
            ours_label,
            theirs_label,
            hunks,
            change_count,
            conflict_count,
            content_hash,
            trailing_newline,
        });
    }

    // Fallback: marker parse of working tree file.
    let workdir = repo
        .workdir()
        .ok_or_else(|| git2::Error::from_str("bare repo has no working directory"))?;
    let raw = std::fs::read_to_string(workdir.join(path))
        .map_err(|e| git2::Error::from_str(&format!("read failed: {e}")))?;
    let trailing_newline = raw.ends_with('\n');
    let parsed = parse_markers(&raw);
    let mut hunks = Vec::new();
    let mut next_id = 0u32;
    for pr in parsed {
        match pr {
            ParsedRegion::Merged { lines } => hunks.push(Hunk {
                id: None,
                kind: HunkKind::Unchanged,
                base_lines: lines.clone(),
                left_lines: lines.clone(),
                right_lines: lines,
                left_line_ops: vec![],
                right_line_ops: vec![],
            }),
            ParsedRegion::Conflict { ours, theirs, base } => {
                let base_lines = base.clone().unwrap_or_default();
                hunks.push(Hunk {
                    id: Some(next_id),
                    kind: HunkKind::Conflict,
                    left_line_ops: line_diff(&base_lines, &ours),
                    right_line_ops: line_diff(&base_lines, &theirs),
                    base_lines,
                    left_lines: ours,
                    right_lines: theirs,
                });
                next_id += 1;
            }
        }
    }
    let change_count = 0;
    let conflict_count = hunks.iter().filter(|h| h.kind.is_conflict()).count() as u32;
    Ok(ConflictDocument {
        path: path.to_string(),
        ours_label,
        theirs_label,
        hunks,
        change_count,
        conflict_count,
        content_hash: format!("{:x}", simple_hash(&raw)),
        trailing_newline,
    })
}
```

Add necessary imports (`build_hunks`, `split_lines`, `Hunk`, `HunkKind`, `line_diff`).

- [ ] **Step 3: Update `document_test.rs` + add blue+red fixture**

In `support.rs`, add:

```rust
/// Base file has line2 shared; main edits line2 (conflict) AND adds a unique
/// left-only line after line1; feature only edits line2 differently.
/// Expected after merge: 1 blue LeftChange (extra line) + 1 Conflict on line2.
pub fn blue_and_red_conflict() -> Fixture {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();

    write(dir.path(), "file.txt", "line1\nline2\nline3\n");
    commit_all(&repo, "base");

    let default_branch = repo.head().unwrap().name().unwrap().to_string();

    repo.branch("feature", &repo.head().unwrap().peel_to_commit().unwrap(), false).unwrap();
    // Or checkout -b feature same as modify_modify_conflict helper style:
    // follow the same pattern as modify_modify_conflict for branch switching.

    // feature: change line2 only
    // ... checkout feature ...
    write(dir.path(), "file.txt", "line1\nFEATURE\nline3\n");
    commit_all(&repo, "feature change");

    // main: change line2 differently + insert BLUE after line1
    // ... checkout default branch ...
    write(dir.path(), "file.txt", "line1\nBLUE\nMAIN\nline3\n");
    commit_all(&repo, "main change");

    // merge feature → conflict
    // ... same merge block as modify_modify_conflict ...
    Fixture { dir, repo }
}
```

Copy the exact branch/merge ceremony from `modify_modify_conflict` — do not invent a different git2 API. Only change the file contents as above.

Update `document_test.rs`:

```rust
use tauri_app_lib::{document, model::HunkKind};

#[test]
fn builds_document_with_one_conflict_hunk() {
    let fx = support::modify_modify_conflict();
    let doc = document::load(&fx.repo, "file.txt").unwrap();
    assert_eq!(doc.path, "file.txt");
    assert_eq!(doc.conflict_count, 1);
    assert!(!doc.content_hash.is_empty());
    let conflict = doc.hunks.iter().find(|h| h.kind == HunkKind::Conflict).unwrap();
    assert!(conflict.left_lines.iter().any(|l| l.contains("MAIN")));
    assert!(conflict.right_lines.iter().any(|l| l.contains("FEATURE")));
}

#[test]
fn blue_and_red_fixture_has_both() {
    let fx = support::blue_and_red_conflict();
    let doc = document::load(&fx.repo, "file.txt").unwrap();
    assert!(doc.change_count >= 1);
    assert_eq!(doc.conflict_count, 1);
    assert!(doc.hunks.iter().any(|h| h.kind.is_blue()));
    assert!(doc.hunks.iter().any(|h| h.kind.is_conflict()));
}
```

Keep the path-rejection tests unchanged (swap only broken imports).

- [ ] **Step 4: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/model.rs src-tauri/src/document.rs src-tauri/tests/support.rs src-tauri/tests/document_test.rs
git commit -m "feat: load conflict documents as stage-based hunks"
```

---

### Task 4: Frontend types + `hunks.ts` decision logic (TDD)

**Files:**
- Modify: `src/types.ts`
- Create: `src/logic/hunks.ts`
- Create: `src/logic/hunks.test.ts`
- Delete (at end of this task or Task 5): `src/logic/panes.ts`, `src/logic/panes.test.ts`

**Interfaces:**
- Produces (TypeScript):
  - `HunkKind`, `Hunk`, `ConflictDocument`
  - `DecisionKind = "pending" | "accepted_left" | "accepted_right" | "keep_base"`
  - `Decisions = Record<number, DecisionKind>`
  - `initDecisions`, `applyDecision`, `applyNonConflicting`, `acceptAllConflicts`
  - `countPendingChanges`, `countPendingConflicts`, `serializeResult`

- [ ] **Step 1: Replace `src/types.ts` region types**

```ts
export type SideStatus = "added" | "modified" | "deleted";

export interface ConflictFile {
  path: string;
  ours_status: SideStatus;
  theirs_status: SideStatus;
  is_binary: boolean;
}

export type LineOp =
  | { op: "equal"; text: string }
  | { op: "insert"; text: string }
  | { op: "delete"; text: string };

export type HunkKind =
  | "unchanged"
  | "left_change"
  | "right_change"
  | "both_same"
  | "conflict";

export interface Hunk {
  id: number | null;
  kind: HunkKind;
  base_lines: string[];
  left_lines: string[];
  right_lines: string[];
  left_line_ops: LineOp[];
  right_line_ops: LineOp[];
}

export interface ConflictDocument {
  path: string;
  ours_label: string;
  theirs_label: string;
  hunks: Hunk[];
  change_count: number;
  conflict_count: number;
  content_hash: string;
  trailing_newline: boolean;
}

export function isBlue(kind: HunkKind): boolean {
  return kind === "left_change" || kind === "right_change" || kind === "both_same";
}

export function isConflict(kind: HunkKind): boolean {
  return kind === "conflict";
}
```

- [ ] **Step 2: Write failing Vitest file `src/logic/hunks.test.ts`**

```ts
import { describe, it, expect } from "vitest";
import {
  initDecisions,
  applyDecision,
  applyNonConflicting,
  acceptAllConflicts,
  countPendingChanges,
  countPendingConflicts,
  serializeResult,
} from "./hunks";
import type { Hunk } from "../types";

const hunks: Hunk[] = [
  {
    id: null,
    kind: "unchanged",
    base_lines: ["line1"],
    left_lines: ["line1"],
    right_lines: ["line1"],
    left_line_ops: [],
    right_line_ops: [],
  },
  {
    id: 0,
    kind: "left_change",
    base_lines: [],
    left_lines: ["BLUE"],
    right_lines: [],
    left_line_ops: [],
    right_line_ops: [],
  },
  {
    id: 1,
    kind: "conflict",
    base_lines: ["line2"],
    left_lines: ["MAIN"],
    right_lines: ["FEATURE"],
    left_line_ops: [],
    right_line_ops: [],
  },
  {
    id: null,
    kind: "unchanged",
    base_lines: ["line3"],
    left_lines: ["line3"],
    right_lines: ["line3"],
    left_line_ops: [],
    right_line_ops: [],
  },
];

describe("hunk decisions", () => {
  it("starts with actionable hunks pending", () => {
    const d = initDecisions(hunks);
    expect(d[0]).toBe("pending");
    expect(d[1]).toBe("pending");
    expect(countPendingChanges(hunks, d)).toBe(1);
    expect(countPendingConflicts(hunks, d)).toBe(1);
  });

  it("serializeResult with all pending is pure base", () => {
    const d = initDecisions(hunks);
    expect(serializeResult(hunks, d, true)).toBe("line1\nline2\nline3\n");
  });

  it("applyNonConflicting all accepts blue left lines", () => {
    let d = initDecisions(hunks);
    d = applyNonConflicting(d, hunks, "all");
    expect(d[0]).toBe("accepted_left");
    expect(d[1]).toBe("pending");
    expect(serializeResult(hunks, d, true)).toBe("line1\nBLUE\nline2\nline3\n");
    expect(countPendingConflicts(hunks, d)).toBe(1);
  });

  it("acceptAllConflicts left resolves red only", () => {
    let d = initDecisions(hunks);
    d = acceptAllConflicts(d, hunks, "left");
    expect(d[1]).toBe("accepted_left");
    expect(d[0]).toBe("pending");
    expect(serializeResult(hunks, d, true)).toBe("line1\nMAIN\nline3\n");
    expect(countPendingConflicts(hunks, d)).toBe(0);
  });

  it("keep_base discards a conflict", () => {
    let d = initDecisions(hunks);
    d = applyDecision(d, 1, "keep_base");
    expect(countPendingConflicts(hunks, d)).toBe(0);
    expect(serializeResult(hunks, d, true)).toBe("line1\nline2\nline3\n");
  });

  it("pending blue does not block apply gating helper", () => {
    let d = initDecisions(hunks);
    d = applyDecision(d, 1, "accepted_left");
    expect(countPendingConflicts(hunks, d)).toBe(0);
    expect(countPendingChanges(hunks, d)).toBe(1);
  });
});
```

- [ ] **Step 3: Run tests — expect fail**

Run: `pnpm test -- src/logic/hunks.test.ts`

Expected: FAIL (module missing)

- [ ] **Step 4: Implement `src/logic/hunks.ts`**

```ts
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
```

- [ ] **Step 5: Run tests — expect pass**

Run: `pnpm test -- src/logic/hunks.test.ts`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/types.ts src/logic/hunks.ts src/logic/hunks.test.ts
git commit -m "feat: add hunk decision logic and types"
```

---

### Task 5: Rebuild `MergeEditor` UI + CSS

**Files:**
- Modify: `src/screens/MergeEditor.tsx`
- Modify: `src/App.css`
- Delete: `src/logic/panes.ts`, `src/logic/panes.test.ts`
- Verify: `src/App.tsx` still compiles (props unchanged: `doc`, `onSave`, `onCancel`)

**Interfaces:**
- Consumes: `doc.hunks`, hunks.ts helpers
- Produces: IntelliJ chrome — toolbar, 3 panes with colors/gutters, footer

- [ ] **Step 1: Replace `MergeEditor.tsx`**

```tsx
import { useMemo, useRef, useState, type UIEvent } from "react";
import type { ConflictDocument, Hunk } from "../types";
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

interface Props {
  doc: ConflictDocument;
  onSave: (content: string) => void;
  onCancel: () => void;
}

function sideLines(hunks: Hunk[], side: "left" | "right"): string[] {
  const out: string[] = [];
  for (const h of hunks) {
    out.push(...(side === "left" ? h.left_lines : h.right_lines));
  }
  return out;
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
                  <button type="button" title="Accept left" onClick={() => setDecisions((d) => applyDecision(d, h.id!, "accepted_left"))}>
                    »
                  </button>
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
                  <button type="button" title="Accept right" onClick={() => setDecisions((d) => applyDecision(d, h.id!, "accepted_right"))}>
                    «
                  </button>
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
```

Remove unused `sideLines` if the final JSX does not call it (or use it). Keep the file free of unused-locals TS errors.

- [ ] **Step 2: Extend `App.css`**

Append (keep existing overview/error rules):

```css
.editor { display: flex; flex-direction: column; height: 100vh; }
.editor-toolbar {
  display: flex; gap: 16px; align-items: center; flex-wrap: wrap;
  padding: 8px 12px; border-bottom: 1px solid #333; background: #252525; color: #ccc;
  font-size: 13px;
}
.toolbar-group { display: flex; gap: 6px; align-items: center; }
.toolbar-status { margin-left: auto; color: #aaa; }
.panes { flex: 1; min-height: 0; display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 2px; }
.pane { overflow: auto; padding: 0; background: #1e1e1e; color: #ddd; font-family: monospace; }
.pane-title { color: #888; padding: 8px; position: sticky; top: 0; background: #1e1e1e; z-index: 1; }
.hunk { display: grid; grid-template-columns: auto 1fr; }
.hunk pre { margin: 0; padding: 0 8px; white-space: pre; }
.hunk.conflict { background: rgba(120, 60, 40, 0.45); }
.hunk.change { background: rgba(40, 80, 140, 0.45); }
.gutter { display: flex; flex-direction: column; gap: 2px; padding: 2px; }
.gutter button {
  font-size: 11px; padding: 0 4px; min-width: 0; line-height: 1.4;
}
.pane.result pre { padding: 8px; margin: 0; white-space: pre; }
```

- [ ] **Step 3: Delete old panes modules; fix compile**

```bash
rm src/logic/panes.ts src/logic/panes.test.ts
pnpm exec tsc --noEmit
pnpm test
```

Expected: PASS (no panes imports left)

- [ ] **Step 4: Commit**

```bash
git add src/screens/MergeEditor.tsx src/App.css
git add -u src/logic/panes.ts src/logic/panes.test.ts
git commit -m "feat: IntelliJ-style merge editor chrome and gutters"
```

---

### Task 6: Update smoke test + end-to-end verification

**Files:**
- Modify: `src-tauri/tests/M1_SMOKE_TEST.md`

- [ ] **Step 1: Rewrite smoke procedure for hunk semantics**

Replace the editor expectations section with:

```markdown
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
```

- [ ] **Step 2: Run automated suites once more**

```bash
pnpm test
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tests/M1_SMOKE_TEST.md
git commit -m "docs: update smoke test for IntelliJ-style merge editor"
```

---

## Self-review (plan vs spec)

| Spec requirement | Task |
|---|---|
| Stage-based hunks / `merge3::build_hunks` | Task 2 |
| Blue + red kinds, ids, line ops | Tasks 1–2 |
| `document::load` via `read_stages` + marker fallback | Task 3 |
| Pure-base Result; pending blues; Apply gates on conflicts | Task 4 |
| Toolbar Apply non-conflicting Left/All/Right | Task 5 |
| Gutters `»` / `X` | Task 5 |
| Stub dropdowns | Task 5 |
| Footer Accept Left/Right, Cancel, Apply | Task 5 |
| Blue+red fixture + smoke update | Tasks 3, 6 |
| No word highlight / no editable Result / no Accept Both | Global constraints |

No TBD/placeholder steps remain. Type names are consistent (`HunkKind` snake_case over the wire, TS string unions matching serde).

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-27-intellij-merge-editor.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
