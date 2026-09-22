---
pipeline_id: debe0710-4fdb-4b4c-a2cd-a56a5d58cf7d
ticket: forge#154 (2953a9bf-11fe-495a-a7d1-f022ae2d1e08) · local docs/planning/tickets/open/TICKET-154-files-tabs.md
aar_id: 89d2ce11-b0ea-4e69-9788-fbe76889f7a0
status: Phase 5 — Complete PASS
title: M9 seq-5 — Files as a left panel + open-file-as-a-CodeView-tab
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/tabs.rs (PURE: TabContent::CodeView, code_view accessors, open_or_switch_code)
  - crates/marley_app/src/app.rs (SHIM: code_view_body + files_panel render moves; open-file→tab; retire tiled panes)
---

## Title
The file tree expands from the LEFT (toggled by 📁, scoped to the active project) and opening a file opens a
full-screen CodeView TAB — never a mid-screen split.

## Scope
### In
- PURE `tabs.rs`: `TabContent::CodeView(CodeViewState)`; `Tab::code_view`/`code_view_mut`; `grid`/`cockpit_section`
  return None for CodeView; `Project::open_or_switch_code(state)` (replace-or-append+activate).
- SHIM `app.rs`: the center render branches Terminal→grid | Cockpit→cockpit_body | CodeView→`code_view_body`
  (moved from the tiled CodeView pane); a LEFT `files_panel` (moved from the tiled FileTree pane) toggled by 📁;
  `open_file_in_viewer` → `open_or_switch_code`; retire `open_files_pane`/`open_code_pane` + the FileTree/CodeView
  arms of the pane `match kind`.

### Out
- Persisting open CodeView tabs / the files-panel toggle. Multi-file tab management (one CodeView tab reused).
- Jump-to-line, editing (the viewer stays read-only — M5's editor is separate).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — one CodeView tab reused: `open_or_switch_code` replaces the existing CodeView tab's content + switches
  (like the old open_code_pane reusing the first CodeView pane), else appends. (Multi-file tabs later.)
- D2 — the Files panel is a LEFT dock/panel (not a tab, not a tiled pane), scoped to `active_project().root`,
  toggled by the 📁 icon / `open-files` action.
- D3 — `terminal_grid_index` already skips non-grid tabs, so a CodeView active tab falls back to the first
  terminal for `workspace()` — no panic (same guard as cockpit tabs).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `Tab::code_view` is called on a CodeView tab it shall return `Some(state)`; on a terminal/cockpit tab, `None`. | unit |
| REQ-002 | WHEN `grid`/`cockpit_section` are called on a CodeView tab they shall return `None`. | unit |
| REQ-003 | WHEN `open_or_switch_code(state)` runs and a CodeView tab exists it shall replace its content + switch; else it shall append one + activate. | unit |
| REQ-004 (visual) | WHEN the 📁 icon is toggled, the file tree shall render as a LEFT panel (not a mid-screen split pane). | driven capture |
| REQ-005 (visual) | WHEN a file is opened, a full-screen CodeView TAB shall fill the center + appear in the rail (not a tiled pane); switching back to a terminal shall render. | driven capture |
| REQ-006 | gate GREEN, cov/MSI 100 on the pure helpers; the render masked. | gate |

## Phase Plan
- **P2** — the pure additions; the two render moves (code_view_body, files_panel); open-file rewire; retire the panes; test plan.
- **P3** — implement.
- **P3.5** — 2-3 critics (render-move fidelity both blocks; open-file/pane-retirement completeness; panic-safety).
- **P4** — pure tests (cov/MSI 100) + driven captures (left files panel; code tab full-screen; terminal returns) + gate.
- **P5** — docs, AAR, archive, close #154.
