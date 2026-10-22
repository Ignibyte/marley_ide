# A project's mark in its notifications and the switcher — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-859-a-project-mark-in-notifications-and-the-switcher.md
- **Pipeline spec:** 859-project-mark-in-notifications-and-switcher.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row (#890, #895 wait on Chad; #900 needs an LLM stand-in).
- **Classification / tier:** feature, M. gpui and gpui_linux, small; Marley crate.
- **Recall (§18.3):** the mark cache keeps the file it read (`Entry.chosen`); notifications post
  from `notifications::post`; #538's scenario has a private notification server. The brain had
  nothing.
- **Decisions:** D1 in the spec.

## Phase 2 — Code
- gpui: `SystemNotification.image: Option<PathBuf>`; `image: None` in gpui's tests and example,
  and in `harness.rs` and `schedule_runs.rs`. gpui_linux sends it as the `image-path` hint.
- `project_icons::logo_file_for(path)`: of the projects whose mark was read, the deepest folder
  holding `path`, and the file its mark came from.
- `notifications::post` takes the view and reads its terminal's working folder for the image,
  never the workspace: a banner can be posted while other entities update.
- `rail_switcher.rs`: `SwitcherEntry` keeps the project's folder (the group's first path, as the
  header's mark reads it); the terminal row draws the mark before the project's name, the thread
  row through `ThreadItem::project_mark`, a slot added to Zed's `ui` crate.
- Gate: the first `just gate-diff` failed the ledger (two paths in one row); split, the second run 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/859-project-mark-in-notifications-and-switcher.sh` (sway, #538's private
notification server, logging each banner's `image-path`).
- `859-01-banner`: the second terminal shows the log's line, "Build|finished|image-path=…/repo/
  logo.png", for the first terminal's notify sent while the second had the focus; the rail's
  header and the title bar show the same logo (REQ-001).
- `859-02-switcher`: the switcher's two terminal rows, each with the logo before "repo"
  (REQ-002).
- `859-03-day`: the same in Marley Day (REQ-002).
- Not shot: a thread row (the run has no thread); it takes the same mark through `ThreadItem`.

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, the ledger rows, look-plan W1.2.
- Brain decision recorded on consultation a802421c.
