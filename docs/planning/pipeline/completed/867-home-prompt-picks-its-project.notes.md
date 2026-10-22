# Home's prompt picks its project — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-867-home-prompt-picks-its-project.md
- **Pipeline spec:** 867-home-prompt-picks-its-project.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "lets finish up the remaining tickets besides the mac issues";
  the Queue's ready row.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #835's prompt starts in Home's workspace; Home's recent rows open with
  `open_workspace_for_paths`, which gives back the workspace. The brain had nothing.
- **Decisions:** D1 in the spec.

## Phase 2 — Code
- `home_page.rs`: `target`, `render_project_chip`, `project_choices`, `worktree_paths`, and
  `start_in`, which both of `start_prompt`'s paths use. Marley crate only, no ledger rows.
- Review: Recent leaves out Home's own project as well as the open ones (the probe showed `repo2`
  twice).
- Gate: `just gate-diff` 17 passed, 0 failed (after the Phase 3 fix too).

## Phase 3 — Test
- Scenario `script/e2e/867-home-prompt-picks-its-project.sh` under sway: `repo` opened once, then
  Marley on `repo2`; stand-in `claude` and `codex` print their folder and task.
- `867-01-menu` (REQ-001): the chip reads "in Home · …/repo2"; its menu shows Home, then `repo`
  under Recent. The open section is empty since the window holds only Home's own project.
- First full run: the started shot was black. Marley panicked opening `repo` from inside Home's
  update (F-claude-867-…). Fixed by opening from a spawned task; the gate ran green again.
- `867-02-started` (REQ-002): `repo` opened as its group in the rail, and its new terminal runs
  the stand-in claude in `…/repo` with "write the README".
- `867-03-day` (REQ-001): Marley Day, Home now in `repo`; the menu lists `repo2` under "Open in
  this window", and Recent is empty (both projects are open).

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (the project chip), look-plan W2.2 already names #867.
- Knowledge: F-claude-867-opening-a-project-from-a-page-read-the-page-001.
- Brain: decided on consultation 4db6bbab66a048e581d86af22633d08d.
