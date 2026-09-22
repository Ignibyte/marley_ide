---
pipeline_id: 6d3239d2-fc4b-40fa-bea0-abbe8bec2a07
ticket: forge#246 (ac5df912-b46a-4dd8-b2af-1f23c2fac7ab) · local docs/planning/tickets/open/TICKET-246-terminal-split-to-file.md
aar_id: 78f856d1-3ec4-4f05-b9db-b866b9b1f0b4
status: Phase 5 — Complete PASS
title: Terminal pane can split-right to a read-only file view (not a new PTY)
type: feature
milestone: M14
references: [forge#237, forge#155, forge#154, forge#198, forge#163]
---

## Title
Let a terminal pane split-right to a READ-ONLY file view (a `PaneContent::CodeView` pane) within ONE tab —
chad's "can you have a terminal open + split right to a file?" — instead of `split_focused` always spawning a
new PTY. The model already supports non-terminal panes; this revives the render + adds a no-PTY split path + a
trigger. (#237 deferred.)

## Scope
### In
- **Revive the CodeView pane render** — add a `CodeView` arm to the session-less pane render (app.rs:~5581,
  today only Git draws) that calls the existing `code_view_body(cv, colors)`.
- **A `split_file_pane(&mut self, path)`** — the existing `open_file_in_viewer` guard ladder (resolve-under-root
  / stat / size / binary → `CodeViewState::new`) ending in `open_pane(Horizontal, After,
  PaneContent::CodeView(cv))` (the shipped no-PTY split) + `persist_grid()`.
- **A trigger** — one command "Split Right → File" that picks a file via the ⌘P finder in a split mode (D-trigger),
  routing the finder's choice to `split_file_pane` instead of the full-screen editor tab.
- **A pure `pane_display_name`** — a CodeView pane's title shows the file BASENAME (not the project root).

### Out (explicitly deferred)
- **Persistence of the split-file pane** — v1 is NON-PERSISTED (the pane drops on restart, matching existing
  FileTree/Git leaves); true path-persistence (`c=<path>` + re-read on restore) is a follow-up (dep #163/#205).
- **Editing** the split file — read-only v1 (reuses `code_view_body`/`CodeViewState`); the editable Buffer rides
  on #242.
- **Split-down / arbitrary-axis file panes**, a keymap/context-menu trigger — optional polish, not v1.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — reuse the shipped no-PTY `open_pane(axis, dir, PaneContent::CodeView(cv))` (workspace.rs:471); do NOT
  touch `split_focused` (the PTY path). The grid model is content-agnostic → resize/focus/close work for free.
- **D2** — reuse the `open_file_in_viewer` guard ladder verbatim (the #190/#196/#106 guards + `CodeViewState::
  new`); the only delta is the terminal call (`open_pane` vs `open_or_switch_code`).
- **D3 (trigger)** — the ⌘P finder in a "split" mode (a `finder_split` flag set by the "Split Right → File"
  command; the finder's existing Enter routes to `split_file_pane` when set), reusing the shipped #198 finder.
  (Design may veto for the active-editor-file source; the finder is the more useful default.)
- **D4 (pure seam)** — `pane_display_name(kind, project_name, code_view_path: Option<&Path>) -> String`: the
  filename for a CodeView pane with a path, else the project name. cov/MSI 100. (Fixes LANDMINE 2 — the title
  hardcoded the project root for every pane.)
- **D5 (non-persist, no-crash)** — v1 does NOT persist the file pane (dropped on restart). VERIFY (design +
  test) that persisting + restoring a grid with a CodeView leaf does NOT crash/malform (the split collapses to
  the terminal) — the #163/#205 restore already drops CodeView leaves; confirm it's graceful.
- **D6 (borrow)** — the render arm uses the `&self` `state().code_view()` accessor (workspace.rs:336), not
  `workspace_mut()`, so `code_view_body(&self)` doesn't fight the borrow checker.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN "Split Right → File" is invoked with a chosen file, the app shall split the focused pane horizontally and render that file read-only in the new pane (via `open_pane` + `code_view_body`), WITHOUT spawning a PTY. | driven + mechanism |
| REQ-002 | A CodeView pane's title shall show the file's basename, not the project root. | pure unit + driven |
| REQ-003 | The split-file pane shall be resizable and closable like any pane (the content-agnostic grid ops). | mechanism + driven |
| REQ-004 | `pane_display_name` shall return the filename for a CodeView pane with a path, and the project name otherwise (Terminal/Git, or no path). | pure unit |
| REQ-005 | Persisting then restoring a grid containing a CodeView pane shall not crash (the pane is dropped; the terminal remains). | unit/integration or mechanism |

## Phase Plan
- **P2 Design** — confirm D1-D6 + the trigger (finder-split mode vs active-file); the manifest (render arm,
  `split_file_pane`, `pane_display_name` + where it's called, the finder route + command); the D5 restore
  no-crash check; `cargo mutants --list`; the test matrix.
- **P3 Implement** — `pane_display_name` (pure) → the render arm → `split_file_pane` → the finder-split trigger.
- **P3.5 Inspect** — critic: the split reuses the guards (no PTY, no unwrap); the D5 restore is graceful; the
  borrow shape; the finder-mode flag is cleared on close (no stuck mode); clean-room.
- **P4 Validate** — pure units (pane_display_name) + the codec/restore no-crash; gate green; DRIVEN (split →
  file beside terminal → resize → close), env-block fallback to units + mechanism if the screen is locked.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #246; archive.
