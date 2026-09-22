---
pipeline_id: 21c19549-d54b-484e-a088-d9f57b88ea82
ticket: forge#56 (1140cdd5-1102-4eb8-a109-c3aad7233c4f) · local docs/planning/tickets/open/TICKET-056-file-tree.md
aar_id: c8e2300f-ce2c-4ae1-95be-244b8947cdfd
status: Phase 5 — Complete PASS
title: Files dock — real project file tree
type: feature
milestone: M2.A
references:
  - crates/marley_project/src/lib.rs (PURE: FileTree + visible_rows + toggle)
  - crates/marley_app/src/app.rs (SHIM: RootView.file_tree + Left-dock render + click-toggle)
  - crates/marley_app/Cargo.toml (add marley_project dep)
---

## Title
The Files dock (empty since M1.B) shows the project's FILE TREE — a pure `FileTree` model built from
the flat file list (#55), rendered as an indented, collapsible tree in the Left dock.

## Scope
### In
- PURE (`marley_project/src/lib.rs`, cov/MSI 100):
  - `pub struct FileTree` (private `roots: Vec<Node>`; private `Node { name, is_dir, collapsed, children }`).
  - `pub struct TreeRow { pub depth: usize, pub name: String, pub is_dir: bool, pub collapsed: bool }`.
  - `FileTree::from_files(files: &[PathBuf]) -> FileTree` — nest by component (non-terminal = dir,
    terminal = file), merge shared prefixes, sort each level DIRS-before-FILES then alphabetical; dir
    nodes start EXPANDED.
  - `pub fn visible_rows(&self) -> Vec<TreeRow>` — pre-order DFS, one row per node (depth from 0); a
    collapsed dir's children are NOT emitted.
  - `pub fn toggle(&mut self, visible_index: usize)` — flip `collapsed` of the DIR at that visible row;
    no-op for a file row / out-of-range.
- SHIM (`app.rs`, `#[cfg_attr(test, mutants::skip)]` + coverage-excluded): `RootView.file_tree` built
  in `new()` (discover Project from cwd → `list_files_in` → `from_files`); the Left dock renders
  `visible_rows()` (indent by depth, ▸/▾ disclosure for dirs, name); clicking a dir row toggles it.
  `dock_panel` takes the rows for the Left dock. `marley_app` gains a `marley_project` dep.

### Out
- Click a FILE → open it (a later ticket / #57 handles open). The Right (Details) dock — #58. Live
  file-watching / refresh — later (the tree is built once at startup). Horizontal scroll of long trees.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — PURE `FileTree` in marley_project (project/files domain, next to `list_files_in`); the render +
  click are the app shim.
- D2 — `visible_index` (a flat row ordinal from `visible_rows`) is the toggle's coordinate — the render
  has row ordinals for free, and it stays in sync with what's shown (collapsed children aren't counted).
- D3 — Dirs sort before files, then alphabetical, at EVERY level (a stable, conventional tree order).
- D4 — Built ONCE at startup (no watcher yet); a collapsed dir hides its subtree in `visible_rows`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `from_files` is given paths sharing a prefix (`a/b/x`, `a/c`), it shall produce ONE `a` dir node containing nested `b` (dir) and `c` (file), merged not duplicated. | unit |
| REQ-002 | WHEN a level holds both dirs and files, `from_files` shall order DIRS before FILES, each group alphabetical. | unit |
| REQ-003 | WHEN `visible_rows` runs and a dir is collapsed, it shall emit the dir row but NOT its descendants; an expanded dir emits its children (depth+1). | unit |
| REQ-004 | WHEN `toggle(i)` targets a dir row, it shall flip that dir's `collapsed`; WHEN it targets a file row or an out-of-range index, it shall do nothing. | unit |
| REQ-005 (visual) | WHEN Marley runs in a project, the Left "Files" dock shall render the project's dirs/files as an indented tree; clicking a dir row shall collapse/expand it. | self-test harness (bundle+drive+capture) |
| REQ-006 | `scripts/gates.sh` GREEN, cov/MSI 100 on the pure FileTree; app shim coverage-excluded. | gate |

## Phase Plan
- **P2** — the `FileTree`/`Node`/`TreeRow` shapes, `from_files`/`visible_rows`/`toggle` algorithms, the
  dock_panel render + click wiring, mutation targets, the unit + self-test plan.
- **P3** — the pure model in marley_project + the app.rs shim (RootView field, new() wiring, Left-dock
  render, click-toggle) + the Cargo dep.
- **P3.5** — critic: the nesting/merge, the per-level sort, collapse-hiding, toggle index mapping +
  dir-only + bounds, mutants; the shim seam (pure vs masked).
- **P4** — pure unit tests (cov/MSI 100) + `-p marley` still green + the SELF-TEST capture (Left dock
  renders the tree; click collapses) + gate GREEN.
- **P5** — docs (arch + CHANGELOG), AAR, archive, close #56.
