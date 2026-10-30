---
pipeline_id: 6459777e-3f6c-477f-8aca-95748b31ab59
ticket: docs/planning/tickets/closed/TICKET-951-open-file-from-the-rail.md
status: Phase 4 — Complete PASS
title: Open File… on the rail's +
type: feature
slice: the Marley-layout shell (docs/marley/workbench-shell.md), the rail
references: []
---

## Title
Open File… on the rail's `+`: any file on disk opens in the project's pane, a Markdown file as its
formatted preview, with a row under the project's Files. Zed scopes a window to its project;
Marley's rail sits above projects, and Chad wants to read a file wherever it lives (2026-10-30).

## Scope
### In
- An Open File… entry on every project header's `+` and its right-click menu (`new_entries`), and
  Open File in Default… on the rail's empty-space menu (`deploy_empty_menu`).
- The chooser is Zed's path prompt through `Workspace::prompt_for_open_path` (files, several at
  once, the project's lister), so `use_system_path_prompts` picks the system dialog or Zed's own.
- Each chosen file opens in the project's workspace, active pane, focused, without joining the
  project's visible folders: a Markdown file as Zed's markdown preview over an editor no pane
  shows (as `terminal_show::open_markdown` does), anything else through `open_abs_path`.
- The rail lists a Markdown preview of a file as a file row under Files, and a file outside the
  project's visible folders names its folder as an absolute path, `~` for the home folder.

### Out (explicitly deferred)
- A file tree inside the rail (Chad chose the chooser).
- Tab tiles, the Ctrl Tab switcher and the bottom row still treat a Markdown preview as a page.
- A remote project's chooser lists the remote machine by construction; no scenario reaches a
  remote host.

## Reference (§20)
Upstream Zed (workspace, markdown_preview): Zed's File ▸ Open File… (`workspace::OpenFiles`) asks
with `prompt_for_open_path_and_open` and routes each path to a window by `WorkspaceMatching`, which
in Marley's one-window multi-workspace could land in another project or a new window. Marley keeps
Zed's chooser and opener (`Workspace::prompt_for_open_path`, `Workspace::open_abs_path` with
`OpenVisible::None`) and only names the workspace: the project whose `+` was pressed. The Markdown
page is Zed's `MarkdownPreviewView`, unchanged. Warp: N/A, an editor behavior.

### Prior art
- Zed's `workspace::OpenFiles` and `prompt_for_open_path_and_open` (crates/zed/src/zed.rs:1103,
  crates/workspace): the chooser, kept; its window routing, not taken.
- `MarkdownPreviewView::open_preview_in_pane` and `open_for_project_path`
  (crates/markdown_preview): a preview over an editor no pane shows. Its `act_as_type` hands out
  the editor, so the rail reads the preview's file with `act_as::<Editor>` and no Zed hunk.
- Marley's own: `terminal_show::open_markdown` (#741's `marley show`), the same opening, taken and
  given a pane argument; `agent_bar::attach` and `agents.rs` use `prompt_for_open_path` with the
  project's lister; the brain's decision "Marley's Attach File types paths through Zed's path
  prompt"; `Project::open_local_buffer` makes a hidden worktree for an outside file and the buffer
  holds it (F-claude-734).
- The rail already lists every center tab with a project path under Files (`member_tabs`,
  rail.rs), outside files included.

## UI proof
Scenario `script/e2e/951-open-file-from-the-rail.sh`, `compositor sway` (it clicks the `+`), with
`use_system_path_prompts` off so Zed's own chooser takes the typed path. Fixtures: a project repo,
and `notes/plan.md` (headings, a list) and `notes/todo.txt` in a folder outside it.
- `951-01-plus-menu.png`: the project's `+` menu with Open File….
- `951-02-markdown.png`: `plan.md` as a formatted page in the pane, its row under Files naming
  the scratch `notes` folder by its absolute path.
- `951-03-text.png`: `todo.txt` in the editor, its row under Files.
- `951-04-empty-menu.png`: the empty-space menu with Open File in Default….
- `951-05-home.png`: a file opened from it in the Default group, its row under Default.

## Locked-In Decisions
- D1 — The chooser is Zed's (`prompt_for_open_path`), so the user's `use_system_path_prompts`
  decides between the system dialog and Zed's typed prompt; `cx.prompt_for_paths` would ignore it.
- D2 — The file opens in the workspace whose `+` was pressed, never by Zed's window routing.
- D3 — An outside file stays out of the project's visible folders (`OpenVisible::None`).
- D4 — Markdown is decided by Zed's language registry (`MarkdownPreviewView::is_markdown_path`),
  not by extension.
- D5 — No Zed hunk: the rail reads a preview's file through `act_as::<Editor>`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project header's `+` is clicked, the rail shall offer Open File… | `951-01-plus-menu.png` |
| REQ-002 | WHEN Open File… is chosen and a Markdown file outside the project is picked, the system shall show it as a formatted page in that project's pane | `951-02-markdown.png` |
| REQ-003 | WHEN a file that is not Markdown is picked, the system shall open it in the editor in that project's pane | `951-03-text.png` |
| REQ-004 | WHILE a file or a Markdown page opened this way is open, the rail shall list it under the project's Files, naming the folder it came from | `951-02`, `951-03` |
| REQ-005 | WHEN the rail's empty space is right-clicked, the menu shall offer Open File in Default…, which opens the picked file in the Default group | `951-04-empty-menu.png`, `951-05-home.png` |
| REQ-006 | WHEN the chooser is dismissed without a file, the system shall open nothing and report nothing | review of the diff |
| REQ-007 | The change shall pass the gate | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — rail.rs entries and `open_files`, `terminal_show::open_markdown` given a pane,
  `member_tabs` for previews and outside folders; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
