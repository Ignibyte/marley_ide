---
pipeline_id: b3d4211f-6e76-4333-941c-1be1251087bb
ticket: docs/planning/tickets/closed/TICKET-898-the-files-button-closes-on-a-second-click.md
status: Phase 4 — Complete PASS
title: The rail's Files button closes the tree on a second click
type: bug
slice: The Marley look, W1.4 (#868)
references: [docs/marley/look-plan.md]
---

## Title
While the project tree shows in the left dock, the rail's Files button closes the dock, wherever
the focus is; while it does not, the button opens it with the focus, as before.

## Scope
### In
- `rail.rs`: `render_files_button`'s click.

### Out (explicitly deferred)
- Ctrl+Shift+E keeps Zed's toggle-focus behaviour, as #868 left it.

## Reference (§20)
Upstream Zed for the dock: `Workspace::toggle_dock`, which the panel buttons use.

### Prior art
- `render_files_button` (#868) dispatches `project_panel::ToggleFocus` by name.
- The brain (consultation 9560d8f8): nothing on this seam.

## UI proof
Scenario `script/e2e/898-files-button-toggles.sh` (sway): a scratch repository with a file open.
- `898-01-open`: the Files button: the tree beside the rail (REQ-001).
- `898-02-closed`: a click in the editor, then the Files button again: the tree gone (REQ-001).

## Locked-In Decisions
- D1 — The button reads the dock's state, as its toggle look already does: open with the project
  panel active means close the left dock; anything else runs `ToggleFocus`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks the rail's Files button while the project tree shows, Marley shall close the tree, wherever the focus is. | `898-01-open`, `898-02-closed` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
