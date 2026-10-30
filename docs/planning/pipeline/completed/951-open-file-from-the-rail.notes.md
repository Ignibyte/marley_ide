# Open File… on the rail's + — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-951-open-file-from-the-rail.md
- **Pipeline spec:** 951-open-file-from-the-rail.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-30: "Can we add a basic file viewer on the left + so i can just open
  a file inside the pane over there? files as in md etc." Offered Open File… on the `+`, Markdown
  formatted, a row under the project; he answered "yep correct. zed is project oriented but marley
  goes up another level where its not. so i want to be able to open files outside a project".
- **Classification / tier:** feature, size S, Marley crate only (no Zed hunk, no ledger row).
- **Recall (§18.3):**
  - F-claude-734: a hidden worktree lives only while something holds it; an outside file's buffer
    holds its own, so the preview's editor keeps the file's worktree alive.
  - F-claude-620: never `open_abs_path` inside the workspace's own update; the menu entry calls
    out of any workspace update (a menu handler is not inside one).
  - L-claude-479: Zed's path prompt honours `use_system_path_prompts`; scenarios 663 and 735 turn
    it off and type the path.
  - L-claude-649: drop the item handle `open_abs_path` returns at once.
  - Brain (consultation 0cbc88fc77374695b7a324caa18f8926): the follow-ups list only; the related
    decision on record is "Marley's Attach File types paths through Zed's path prompt".
- **Discovery** (an Explore pass and reads):
  - `rail.rs` `new_entries` (~6313) builds a header's `+` and right-click menu; `deploy_empty_menu`
    (~4875) the empty space's, each entry wrapped in `in_home`.
  - `member_tabs` (rail.rs ~11036) lists every center tab but terminals, browsers and thread tabs;
    a Singleton item with a project path is `TabKind::File` under Files (`files_rows`,
    marley_rail.rs:1525), its folder the path's parent inside the worktree. An outside file's
    hidden worktree is rooted at the file, so its folder comes out empty today.
  - `MarkdownPreviewView` has no project path (TabKind::Other) but `act_as_type::<Editor>` returns
    its editor.
  - `terminal_show::open_markdown` (terminal_show.rs:180) opens a preview of a path in a given
    pane through `open_local_buffer` (local and remote: `find_or_create_worktree`).
- **Decisions:** D1–D5 in the spec.

### Design
- **rail.rs** (Marley crate):
  - `new_entries`: Open File… after New Browser Tab, calling `rail.open_files(&workspace, …)`.
  - `deploy_empty_menu`: Open File in Default… after New Browser Tab in Home, through `in_home`.
  - `fn open_files(&self, workspace, window, cx) -> anyhow::Result<()>`: `activate_workspace`, then
    `prompt_for_open_path(PathPromptOptions { files: true, directories: false, multiple: true,
    prompt: Some("Open") }, DirectoryLister::Project(project), …)`; a `spawn_in` awaits it; `None`
    (dismissed) returns quietly; an error goes to the log and a toast through
    `detach_and_prompt_err`; each path is Markdown (`is_markdown_path` with the project's
    languages) → `terminal_show::open_markdown_in(workspace, path, active pane)`, else
    `open_abs_path(path, OpenVisible::None, focus)`, handle dropped.
  - `member_tabs`: a tab's path is its own project path, else for a `MarkdownPreviewView` its
    `act_as::<Editor>` editor's; and when the path's worktree is not visible, the folder is the
    file's absolute parent with the home folder as `~` (`util::paths::home_dir`-based, as the rail
    tildes elsewhere).
- **terminal_show.rs** (Marley crate): `open_markdown` split so a `pub(crate) fn
  open_markdown_in(workspace, path, pane, window, cx)` takes the pane; the terminal path passes
  the terminal's pane as now.
- **Docs at Complete:** CHANGELOG, guide.md (the rail's `+`), docs/marley_architecture/
  marley_workbench.md, the walkthrough stop and the Feature Map card.

### Visual check plan
| REQ | Scenario does | Shot |
|---|---|---|
| 001 | clicks the project's `+` | `951-01-plus-menu.png` |
| 002, 004 | Open File…, types `$E2E_WORK/notes/plan.md`, Enter | `951-02-markdown.png`: headings formatted, row under Files with the folder |
| 003, 004 | Open File… again, `notes/todo.txt` | `951-03-text.png`: editor tab, row under Files |
| 005 | right-clicks empty rail space, Open File in Default…, `notes/todo.txt` | `951-04-empty-menu.png`, `951-05-home.png` |
| 006 | review of the diff (a dismissed prompt returns `Ok(None)`) | — |
Not reachable: the system file dialog (a portal outside the compositor; the setting picks it) and
a remote project's chooser.

### Risks
- The preview's tab is the only thing holding the editor and so the buffer and hidden worktree;
  closing the tab frees them, which is the intent.
- Typing a path into Zed's prompt in sway: 735 does it; the prompt may need the path's folder
  typed first and the file chosen from the list.
