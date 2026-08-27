# IntelliJ-style 3-pane merge editor

Date: 2026-08-27  
Status: draft for review  
Scope: replace the M1 marker-based merge editor with a stage-based, IntelliJ-like 3-way merge UI

## Goal

Bring `MergeEditor` up to the IntelliJ 3-way merge interaction model shown in the reference screenshot:

- Left = ours, Center = Result, Right = theirs
- Red blocks = conflicts; blue blocks = non-conflicting changes
- Per-hunk gutter Accept (`»`) and Discard (`X`)
- Toolbar: Apply non-conflicting Left / All / Right
- Status: `N change(s). M conflict(s).`
- Footer: Accept Left / Accept Right / Cancel / Apply

Success means a user can open a conflicted file, see both blue and red hunks against pure base in Result, resolve them with gutters/toolbar/footer, and Apply a clean file with no conflict markers.

## Current state (M1)

- Backend `document::load` parses working-tree conflict markers into `Region::Merged | Region::Conflict`.
- `git::read_stages` already returns base / ours / theirs blobs but is unused by the editor path.
- Frontend `MergeEditor` renders three plain `<pre>` panes from `serializeResult`; Accept Left/Right only; Result is read-only; no hunk highlighting or gutters.
- Decision logic lives in `src/logic/panes.ts` (conflict-id → accept ours/theirs/both/manual).

## Decisions (locked)

| Topic | Choice |
|---|---|
| Visual + interaction parity | Full chrome parity with the screenshot |
| Non-conflicting changes | First-class blue hunks + Apply Left/All/Right |
| Result on open | Pure base (stage 1); blues pending until applied |
| Result editing | Actions only (no typing this pass) |
| Gutter `X` | Keep base for that hunk; mark resolved |
| Highlighting | Line/block only (no word-level) |
| Approach | Stage-based 3-way hunk model (Approach 1) |

## Out of scope

- Word-level / intra-line diff highlighting
- Editable Result / freehand typing
- Working “Ignore whitespace” / “Highlight words” dropdown behavior (UI stubs only)
- Binary-file 3-way view (keep today’s binary refusal / overview behavior)
- Overview screen redesign

## Architecture

```
read_stages(base, ours, theirs)
        │
        ▼
   merge3::build_hunks   ──►  ConflictDocument { hunks, counts, labels, hash }
        │
        ▼
   MergeEditor (React)
        │  decisions: hunkId → pending | accepted_left | accepted_right | keep_base
        ▼
   serializeResult(hunks, decisions)  ──►  save_resolution
```

Marker parsing remains as a **fallback** when stages are incomplete (e.g. missing base). Prefer stages whenever all three (or the two that exist for add/add) are available.

## Data model

### Backend (`src-tauri/src/model.rs`)

Replace editor-facing `regions: Vec<Region>` with ordered `hunks: Vec<Hunk>`.

```rust
enum HunkKind {
    Unchanged,   // identical on all sides present
    LeftChange,  // ours ≠ base, theirs == base  (blue)
    RightChange, // theirs ≠ base, ours == base  (blue)
    BothSame,    // ours == theirs, both ≠ base  (blue)
    Conflict,    // ours ≠ theirs (and at least one ≠ base) (red)
}

struct Hunk {
    id: u32,                 // stable within the document; only on actionable kinds
    kind: HunkKind,
    base_lines: Vec<String>,
    left_lines: Vec<String>, // ours
    right_lines: Vec<String>, // theirs
    left_line_ops: Vec<LineOp>,   // base → left (for line highlighting)
    right_line_ops: Vec<LineOp>,  // base → right
}

struct ConflictDocument {
    path: String,
    ours_label: String,
    theirs_label: String,
    hunks: Vec<Hunk>,
    change_count: u32,    // total blue hunks at load (LeftChange | RightChange | BothSame)
    conflict_count: u32,  // total Conflict hunks at load
    content_hash: String, // hash of stage inputs used for load (see below)
}
```

Notes:

- `Unchanged` hunks may omit `id` (or use a sentinel unused by decisions). Prefer `Option<u32>` / only assign ids to actionable hunks.
- `BothSame` is blue: either side’s Accept applies the same lines; Discard keeps base.
- Preserve whether the original blobs ended with a trailing newline when serializing.
- Toolbar status text uses **pending** counts from frontend decisions (`countPendingChanges` / `countPendingConflicts`), not the static `change_count` / `conflict_count` fields on the document. Those document fields are totals at load for convenience/diagnostics.

### Frontend (`src/types.ts`)

Mirror the Rust shape. Replace `Region` usage in the editor path with `Hunk`.

### Decisions (`src/logic/hunks.ts`)

New module (migrate/replace `panes.ts`):

```ts
type DecisionKind = "pending" | "accepted_left" | "accepted_right" | "keep_base";
type Decisions = Record<number, DecisionKind>;
```

- `initDecisions(hunks)` → all actionable hunks `pending`
- `applyDecision(decisions, id, kind)`
- `applyNonConflicting(decisions, hunks, side: "left" | "right" | "all")` — only blue hunks still `pending`
- `acceptAllConflicts(decisions, hunks, side: "left" | "right")` — only red hunks still `pending`
- `countPendingChanges` / `countPendingConflicts`
- `serializeResult(hunks, decisions)`:
  - `unchanged` → `base_lines` (same as left/right)
  - `pending` blue → `base_lines`
  - `pending` red → `base_lines` (Result stays pure base; do **not** re-emit markers into Result)
  - `accepted_left` → `left_lines`
  - `accepted_right` → `right_lines`
  - `keep_base` → `base_lines`
- Apply enabled iff `countPendingConflicts === 0`

Unresolved red hunks staying as base in Result (rather than markers) matches “Result starts as pure base” and “actions only.” The user must Accept or `X` every conflict before Apply.

## Backend changes

### New: `src-tauri/src/merge3.rs`

Pure function:

```rust
pub fn build_hunks(base: &[String], ours: &[String], theirs: &[String]) -> Vec<Hunk>
```

Algorithm sketch (line-based):

1. Diff base→ours and base→theirs (reuse `diff::line_diff` / `similar`).
2. Walk aligned spans; classify each span into one of the `HunkKind`s.
3. Assign monotonic ids to actionable hunks.
4. Attach per-side `LineOp`s for highlighting.

Edge cases to cover in unit tests:

| Case | Expected |
|---|---|
| Identical files | single `Unchanged` |
| Left-only edit | `LeftChange` blue |
| Right-only edit | `RightChange` blue |
| Same edit both sides | `BothSame` blue |
| Different edit same span | `Conflict` red |
| Insert left only / right only | blue add hunks |
| Delete left only / right only | blue delete hunks |
| Delete vs modify | `Conflict` |
| Empty base (add/add) | no base lines; equal adds → `BothSame`, unequal → `Conflict` |
| Missing ours or theirs stage | treat missing side as empty file |

### `document.rs`

1. Call `git::read_stages`.
2. If usable stages present → split into lines → `merge3::build_hunks` → document.
3. Else fall back: parse markers; synthesize hunks (`Region::Merged` / `ParsedRegion::Merged` → `Unchanged`, `Conflict` → `Conflict` with base if present).
4. `content_hash`: hash a canonical concatenation of the stage texts actually used (or fallback raw file), so external-change detection still works.
5. Labels still from `git::branch_labels`.

### `commands` / IPC

Same `load_conflict` / `save_resolution` commands; only the document payload shape changes. Update TS `api.ts` / `types.ts` accordingly. Do not rename the IPC methods in this pass.

## UI

### Toolbar

```
Apply non-conflicting changes:  [» Left]  [» All]  [« Right]
[Do not ignore ▾]   [Highlight words ▾]              N change(s). M conflict(s).
```

- Left / All / Right call `applyNonConflicting`.
- Dropdowns render but only the default options work (`Do not ignore`, `Do not highlight`); no behavior change this pass.
- Counts = **pending** blues and reds (update live as decisions change).

### Panes

| Left | Center | Right |
|---|---|---|
| `Changes from {ours_label}` | `Result` | `Changes from {theirs_label}` |

- Monospace, line numbers, synchronized vertical scroll.
- Row background: red for `Conflict`, blue for blue kinds, none for `Unchanged`.
- Line-ops drive insert/delete styling inside changed rows (block/line level only).
- Result is read-only and always `serializeResult` output.

### Gutters

Between Left↔Result and Result↔Right, on each actionable hunk:

- `»` — accept that side for the hunk (`accepted_left` / `accepted_right`)
- `X` — `keep_base`

For `BothSame`, accepting either side is equivalent; UI may show both gutters.

### Footer

- **Accept Left** / **Accept Right** — `acceptAllConflicts` (red only)
- **Cancel** — back to overview, no write
- **Apply** — `serializeResult` → `save_resolution`; disabled while any conflict is `pending`

Pending blues do **not** block Apply: they remain base in the written file (same as `X`). That matches “discard = keep base” and avoids trapping users who only care about conflicts. Toolbar Apply-non-conflicting is how blues get into Result.

## Migration

1. Add `merge3.rs` + tests; keep marker path working.
2. Switch `document::load` to stages→hunks; update `ConflictDocument` serde.
3. Add `src/logic/hunks.ts` (+ tests); delete or thin-wrap `panes.ts`.
4. Rebuild `MergeEditor` UI (toolbar, gutters, colors, counts).
5. Update `M1_SMOKE_TEST.md` for new open/resolve expectations.
6. Add fixture with **both** a blue hunk and a red conflict in one file (extend `support.rs`).

## Testing

**Rust**

- `merge3` unit tests for every edge case table row above.
- `document` integration test on `modify_modify_conflict` (today’s fixture → 1 red hunk) and new blue+red fixture.
- Fallback path: load still works if stages missing (synthetic marker input if needed).

**TypeScript**

- `hunks.test.ts`: init, accept, discard, bulk non-conflicting, accept-all conflicts, serialize, Apply gating (`pending` conflict blocks; pending blue does not).

**Manual**

Update smoke test:

1. Fixture with left-only blue change + modify/modify red conflict.
2. Open editor → Result shows base; `1 change. 1 conflict.` (or counts matching fixture).
3. `» All` non-conflicting → blue applied; conflict still pending; Apply still disabled.
4. Gutter Accept on conflict → Apply enabled.
5. Apply → overview clear; on-disk content matches decisions.

## Risks

- **3-way alignment bugs** (adjacent inserts/deletes) — mitigate with focused `merge3` tests and a real git fixture.
- **API break** for `ConflictDocument` — acceptable; only this app consumes it.
- **Trailing newline / empty last line** — pin behavior with serialize tests.
- **Users expecting auto-applied blues** — documented decision: pure base on open; toolbar applies blues.

## Open points explicitly closed

- Word highlight dropdown: stub only.
- Ignore-whitespace dropdown: stub only.
- Accept Both (ours then theirs) for a single conflict: not in the IntelliJ chrome for this pass; user picks Left, Right, or Keep base. Can return later if needed.
