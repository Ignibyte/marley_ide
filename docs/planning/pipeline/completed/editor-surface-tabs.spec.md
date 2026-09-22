---
pipeline_id: 4385bf63-1409-42ec-b23b-a0122982752c
ticket: forge#237 (2d01685e-ac8a-4b73-a2c7-d4233b279b88) · local docs/planning/tickets/open/TICKET-237-editor-surface-tabs.md
aar_id: 57916c99-669d-42de-ae86-b20adebe9d0a
status: Phase 5 — Complete PASS
title: Editor-surface tabs — files open as editor tabs within one surface, not N rail rows
type: feature
milestone: M13
references: [forge#233, forge#236, forge#164, forge#154]
---

## Title
chad feedback #9 (anti-clutter): "avoid each time you open something it clutters the tab on the left…
all files under a single pane." Today EVERY file-open (⌘P finder, Files-tree click, link/search/diff)
funnels through `open_file_in_viewer` → `Project::open_or_switch_code`, which adds ONE read-only
`TabContent::CodeView` TAB per file — and `rail_rows` projects every tab into a rail row, so N open
files = N rail rows (the clutter). #237 introduces an EDITOR SURFACE: the `CodeView` tab is recast to
hold a MULTI-FILE `EditorSurface` (a path-keyed set of file tabs + an active one), so N open files land
in ONE surface (ONE rail row) with N file tabs INSIDE it (the VSCode model). Read-only (reuses the
existing `CodeViewState` doc model + `code_view_body` render). Editing (the `Buffer` core) and
terminal-split-to-file are deferred.

## Scope
### In
- **Pure `editor_surface::EditorSurface`** (cov/MSI 100) — `{ files: Vec<CodeViewState>, active: usize }`
  with `open(state)` (dedupe by path → switch to the existing tab & refresh, else append + activate),
  `close(idx)` (remove + clamp active; `None` when it drains the last file), `activate(idx)`,
  `active_file()`, `files()`, `len()`. Mirrors `open_or_switch_code`'s dedupe (#164) but INSIDE one tab.
- **tabs.rs:** recast `TabContent::CodeView(CodeViewState)` → `TabContent::Editor(EditorSurface)`; the
  `Tab` accessors follow (`editor()`/`editor_mut()`; a convenience `active_code_view()` for the render);
  `Project::open_or_switch_code` → `open_or_switch_editor(state)` — find the project's ONE editor tab
  (create it if absent) then `surface.open(state)` (so a 2nd file joins the surface, not a new tab).
- **app.rs shim:** `open_file_in_viewer` routes to `open_or_switch_editor`; the editor tab renders a
  horizontal FILE-TAB STRIP (each file's name, click → activate, × → `close`) above the active file's
  `code_view_body`; the persistence `TabLayout` maps the editor tab (serialize its file paths, restore by
  re-reading — OR skip if that proves large; decided at design).

### Out (deferred — flagged to chad)
- **Editing** — v1 is READ-ONLY (reuses `CodeViewState`). The editable `marley_editor::Buffer` (ropey,
  `EditOrigin::Human|Agent`) is the reuse core, but file-load/save + multi-buffer wiring is greenfield.
- **Terminal-split-to-file** (chad's "terminal + split-right to a file") — needs the retired
  `PaneContent::CodeView` pane render revived (app.rs:5358) + a split path calling `open_pane` (not
  `split_focused`, which spawns a PTY). Its own ticket.
- Editor-surface PERSISTENCE across restart if it proves large (v1 may be transient — the boot restore
  already drops the retired file panes; decided at design).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — recast the existing CodeView tab into a multi-file editor surface** (not a brand-new
  TabContent variant alongside it) — reuses the render/persistence/rail plumbing; the `PaneContent::CodeView`
  (pane) variant is untouched (reserved for the deferred split-to-file).
- **D2 — read-only v1** (reuse `CodeViewState` + `code_view_body`); editing deferred (`Buffer` greenfield).
- **D3 — one editor surface per project** (chad's "all files under a single pane") — `open_or_switch_editor`
  finds/creates the single editor tab; every file-open lands there.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `EditorSurface::open(state)` shall switch to (and refresh) an existing file tab of the same path, else append the file + activate it; `close` shall remove + clamp the active index (None when empty). | editor_surface unit tests, cov/MSI 100 |
| REQ-002 | Opening a file (any entry point) shall route into the project's single editor surface (creating it once), NOT add a new rail Tab row per file. | shim review + driven capture (open 3 files → 1 rail row) |
| REQ-003 | The editor tab shall render a file-tab strip (each open file, click to activate, × to close) above the active file's content. | shim review + driven capture |
| REQ-004 | Opening N distinct files shall yield ONE editor surface with N file tabs; the left rail shall show one editor row, not N. | driven capture (3 files → 3 file tabs, 1 rail row) |

## Phase Plan
- **P2 Design** — the `EditorSurface` API + tests; the `TabContent::CodeView → Editor` recast + the
  accessor/`open_or_switch` changes; the render (tab strip + body); the persistence decision (paths vs
  skip); run `cargo mutants --list`.
- **P3 Implement** — pure (`editor_surface.rs`) → tabs.rs recast → app.rs routing + render.
- **P3.5 Inspect** — critics: the surface dedupe/close correctness, the recast completeness (all CodeView
  sites updated), the render, persistence/no-panic, clean-room.
- **P4 Validate** — RUN the pure tests; gate green; DRIVEN capture (open 3 files → 1 editor surface + 3
  file tabs, rail unchanged).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #237; the train COMPLETE → report + offer push.
