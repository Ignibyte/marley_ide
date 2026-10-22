# The rail's Files button closes the tree on a second click — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-898-the-files-button-closes-on-a-second-click.md
- **Pipeline spec:** 898-files-button-toggles.spec.md

## Phase 1 — Plan
- **Request:** Chad's report, 2026-10-22, taken before #891 (paused in Plan).
- **Classification / tier:** bug, S. Marley crate only.
- **Recall (§18.3):** #868 built the button on `project_panel::ToggleFocus`; `ToggleFocus` closes
  only a focused panel, and a click on the rail moves the focus. The brain had nothing.
- **Design:** `render_files_button` computes `shown` already; its click closes the left dock with
  `Workspace::toggle_dock(DockPosition::Left, …)` while shown, else dispatches `ToggleFocus`.
- **File manifest:** `rail.rs`; the scenario.

## Phase 2 — Code
- `render_files_button`: the click closes the left dock with `Workspace::toggle_dock` while the
  tree shows (the same `shown` its toggle look reads), else dispatches `ToggleFocus` as before.
  `DockPosition` comes from `workspace::dock`.
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/898-files-button-toggles.sh` (sway).
- `898-01-open`: the Files button: the tree (repo, src, lib.rs, README.md) beside the rail, the
  button lit (REQ-001).
- `898-02-closed`: a click in the editor (the caret there), then Files again: the tree gone, the
  editor where it was (REQ-001).
- Not shot in Day: nothing drawn changed.

## Phase 4 — Complete
- CHANGELOG (Fixed), `marley_workbench.md`'s #868 note.
- Knowledge: F-claude-898-toggle-focus-only-closes-a-focused-panel-001.
- Brain decision recorded on consultation 9560d8f8.
