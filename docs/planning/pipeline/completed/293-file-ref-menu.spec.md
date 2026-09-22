---
pipeline_id: 6d232601-2d36-4732-a1ad-93f79f8e5da5
ticket: forge#293 (05619daf-a14a-4f7a-8b32-c360155f2815) · local docs/planning/tickets/open/TICKET-293-file-ref-menu.md
aar_id: 6d5fbb36-903b-4be8-bcfe-50bcaef68054
status: Phase 5 — Complete PASS
title: File-ref context menu in terminal output — open / split / reveal-in-tree / copy-path
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
Right-click a `path:line` file ref in block output → a 4-item context menu (Open in
editor · Open in split-right · Reveal in file tree · Copy path), broadening the #212
single left-click→open. Reuses the #166 menu infra + #212/#246 openers + a new pure
`FileTree::reveal`.

## Scope
### In
- `context_menu.rs` (pure): `MenuAction::{OpenInEditor, OpenInSplit, RevealInTree, CopyPath}`;
  `MenuKind::FileRef` (a payload-less marker — `MenuKind` is `Copy`; the ref target lives in the
  shim); `FILE_REF_MENU_ITEMS: [_; 4]`; an `items_for(MenuKind::FileRef)` branch → that table.
- `marley_project` (pure): `FileTree::reveal(&mut self, path: &Path) -> Option<usize>` — expand every
  ancestor directory of `path`, then return `path`'s visible-row index (after expansion); `None` when
  `path` is not in the tree (outside the project / stale).
- `app.rs` (shim): a `RootView` field `file_ref_menu_target: Option<(PathBuf, Option<usize>, Option<usize>)>`
  set when the menu opens; a RIGHT-click on a `LinkTarget::File` link segment (mirroring the #212
  left-click hit-test at ~8041) opens `ContextMenuState` with `MenuKind::FileRef` at the cursor; the 4
  dispatches — `open_file_at` (#212) / `split_file_pane` (#246) / `file_tree.reveal` → `files_scroll` /
  `write_to_clipboard(path string)`.

### Out (explicitly deferred)
- **A persistent reveal HIGHLIGHT** (a selected-row marker in the tree) — reveal-scroll (bring into
  view) is the core; a persistent highlight needs a tree-selection state + render, a follow-up.
- **Disabled-item states** — the 4 items always render; actions no-op gracefully when the file isn't
  openable (the #212/#246 stat-before-read guards) or isn't in the tree (`reveal` → `None`). No
  greyed-item machinery.
- **A modifier-click variant** (⌘-click etc.) — only the right-click opens the menu; a modifier variant
  is a follow-up.
- A menu on a `LinkTarget::Url` (web links) — the menu is file-ref-only; a URL menu (Open / Copy URL)
  is a follow-up.

## Reference (§20)
The IDE / terminal file-ref right-click menu (VS Code & Zed integrated terminal: right-click a linked
path → Open, Reveal in Explorer/Finder, Copy Path). Marley matches with its own 4-item menu on a
linkified `path:line`. Clean-room §20: Marley's own `ContextMenuState`/`MenuKind` (#166) + `FileTree`
+ the shipped openers; no Zed/Warp source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — `MenuKind::FileRef` is a payload-less marker; the target lives in a shim field.** `MenuKind`
  is `Copy` and a file ref carries a `PathBuf` (not `Copy`); rather than drop `Copy` (ripples through
  every by-value use), the shim stores `file_ref_menu_target` set atomically at menu-open (the modal
  menu keeps it stable — the #175 "bound here" intent, via the shim field).
- **D2 — the menu opens ONLY on a `LinkTarget::File` right-click.** A File link always has a path, so
  all 4 items are always valid; no resolve-gate / disabled state. A right-click NOT on a File link
  keeps the existing #175 Block / #166 Split menu.
- **D3 — `reveal` = expand ancestors + return the row index; the shim scrolls `files_scroll`.** Pure
  tree math in `marley_project`; the scroll + repaint is the shim. `None` (not in tree) → a no-op.
- **D4 — reuse the shipped openers verbatim** — `open_file_at` (#212), `split_file_pane` (#246),
  `resolve_under_root` (#190), `write_to_clipboard`. #293 adds only the menu + `reveal`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `items_for(MenuKind::FileRef)` shall return `[OpenInEditor, OpenInSplit, RevealInTree, CopyPath]` in that order. | context_menu.rs unit. |
| REQ-002 | WHEN `path`'s ancestor dirs are collapsed, `FileTree::reveal(path)` shall expand them and return `path`'s post-expansion visible-row index; WHEN `path` is not in the tree, it shall return `None`. | marley_project unit: a nested tree, collapse an ancestor, `reveal` → expanded + the right index; a foreign path → `None`. |
| REQ-003 | WHEN a `LinkTarget::File` ref in block output is right-clicked, the shim shall open the FileRef context menu bound to that ref. | Live drive: right-click a `path:line` → the 4-item menu appears. |
| REQ-004 | WHEN "Open in split-right" is chosen, the shim shall open the ref's file in a split pane (#246). | Live drive: choose it → the file opens in a right split. |
| REQ-005 | `items_for(FileRef)` + `FileTree::reveal` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — the `MenuAction`/`MenuKind::FileRef`/`FILE_REF_MENU_ITEMS`/`items_for` additions; `FileTree::reveal` algorithm; the shim (the right-click hit-test + the target field + the 4 dispatches); the test plan.
- **P3 Implement** — the pure additions + the shim wiring.
- **P3.5 Inspect** — the reveal ancestor-expansion + the not-in-tree None; the right-click hit-test (only a File link opens FileRef, else the Block/Split menu); the target-field lifecycle (set/clear); the item-table order.
- **P4 Validate** — pure units (REQ-001/002) + a LIVE DRIVE (right-click a `path:line` → the menu → Open in split-right → a split; Reveal → the tree scrolls); gate green [diff].
- **P5 Complete** — CHANGELOG + app_shell.md, AAR, close #293.
