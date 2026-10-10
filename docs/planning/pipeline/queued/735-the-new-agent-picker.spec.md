---
pipeline_id: 124e0ebc-5072-4db8-960e-b73f5aea9062
ticket: docs/planning/tickets/open/TICKET-735-the-new-agent-picker.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The New Agent picker
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/queued/734-an-agent-thread-starts-in-a-tab.spec.md
---

## Title
`marley: new agent` asks where as well as which agent: a thread in a tab (#734) or an agent CLI in
a terminal, started in a folder you pick, wherever you are. Chad, 2026-10-10: "it should auto
detect what project you are in or you need to Open Agent In Path or something like that. Or open
agent in a path finder."

## Scope
### In
- The existing `marley::NewAgent` picker (`agents.rs`, `NewAgentPicker`) gains a second step,
  Where, after the agent is chosen:
  - **the guess**, first: the active editor's project folder, else the active terminal's folder,
    else the shown project's root, else the home folder;
  - the window's open projects, then recent local projects (`WorkspaceDb::recent_project_workspaces`);
  - **Browse…**: Zed's path prompt (`Workspace::prompt_for_open_path`, local lister); a file picked
    there means its folder.
- A thread choice opens #734's tab in the shown group on that folder; a CLI choice opens a new
  terminal of the shown group in that folder (`agents::start_in_terminal` with the directory, as
  `launch.rs` does).
- **Open Agent Here…**: on a terminal's rail row (its folder), and on a folder in the project panel
  (a Zed touch: one menu entry and a workspace action). Both open the picker with Where already
  answered.
- **New Agent…** heads the agent part of every `+` menu in the rail, and Home's New Agent card gets
  it too; Home's line "Agent Panel threads need a project folder" goes.
- `docs/marley/guide.md`: the picker, Open Agent Here.

### Out (explicitly deferred)
- Remote hosts in Where: #741 and #742.
- A permission mode chosen per folder: the CLI's mode still comes from the shown project
  (`agents::launch_mode`), as #532 set it.

## Reference (§20)
Upstream Zed: `workspace`'s path prompt (`Workspace::prompt_for_open_path`, drawn by
`open_path_prompt::OpenPathPrompt` when native dialogs are off) for Browse…, and `project_panel`'s
Open in Terminal (`open_in_terminal`: the selected entry, or a file's folder, dispatched as a
workspace action) for Open Agent Here. Behavior kept: Zed's own prompt and its own folder rule.

### Prior art
- **The code we ship:**
  - `agents.rs`: `NewAgentPicker` / `NewAgentDelegate` (#450's picker: Zed's agents, then the CLIs
    on the search path), `start_in_terminal(workspace, directory, kind, input, joining, …)`
    (`agents.rs:410`) and `launch_input`, used with a directory by `launch.rs:433-452`.
  - `Workspace::prompt_for_open_path(PathPromptOptions, DirectoryLister, …) ->
    oneshot::Receiver<Option<Vec<PathBuf>>>` (`workspace.rs:3443`), already called by Marley in
    `agent_bar.rs:58` and `rusty/import.rs:49`.
  - `WorkspaceDb::recent_project_workspaces` (`workspace/src/persistence.rs:2109`), which Home's
    Recent Projects reads (`home_page.rs:106`).
  - `project_panel.rs` `deploy_context_menu` (1119) and `open_in_terminal` (4143): no hook for
    another crate's entry, so Open Agent Here is three small hunks there.
  - `TerminalView` / `Terminal::working_directory` for a terminal's folder.
- **Behavior maps:** `docs/orca_architecture/01-agents-and-sessions.md` §2.1: Orca's launch takes
  the worktree root, or a recorded `cwd` that forces a terminal launch; Marley asks instead, and
  keeps the thread view with #734's hidden worktree.
- **Published material:** none beyond ACP's `session/new` `cwd`.

## UI proof
`script/e2e/735-the-new-agent-picker.sh`, under `compositor sway` (the rail's rows and the project
panel take clicks). A fake `claude` on the PATH logs its working folder; the Marley entry runs the
scripted agent, which logs `session/new`'s `cwd`.

Shots:
- `735-01-where`: after choosing Claude Code (CLI) with a terminal active in `repo/sub`, the Where
  step with the guess `repo/sub` first, the open project, a recent project and Browse….
- `735-02-browse`: Browse… → `other/` confirmed → a new terminal in the shown group, the fake
  `claude` printing `cwd: …/other`.
- `735-03-thread`: Marley (thread) with the guess → a thread tab in the shown group; the agent log's
  `cwd` is the guessed folder.
- `735-04-here-terminal`: a terminal row's Open Agent Here… → the picker shows agents only; the
  chosen CLI starts in that terminal's folder.
- `735-05-here-panel`: a folder's Open Agent Here in the project panel → the same, in that folder.

## Locked-In Decisions
- **D1:** one picker, two steps (agent, then where), extending #450's picker rather than a second
  one.
- **D2:** the guess is the first row, so Enter twice starts the agent where you are.
- **D3:** Browse… uses Zed's own prompt (native or Zed's picker, as Zed's setting says); a file
  picked means its folder.
- **D4:** Open Agent Here in the project panel is a Zed touch of three hunks (an action carrying
  the folder, a menu entry for folders, its handler), each with a ledger row.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent is chosen in the New Agent picker, the picker shall ask where, listing the guess first, then the open projects, recent projects and Browse…. | Shot 735-01 |
| REQ-002 | WHILE a terminal is the active item, the guess shall be that terminal's folder. | Shot 735-01 |
| REQ-003 | WHEN Browse… confirms a folder for an agent CLI, the system shall start the CLI in a new terminal of the shown group, in that folder. | Shot 735-02 and the fake CLI's log |
| REQ-004 | WHEN a thread agent is chosen with a folder, the system shall open its thread in a tab of the shown group, with its session in that folder. | Shot 735-03 and the agent log |
| REQ-005 | WHEN Open Agent Here… is chosen on a terminal's rail row, the picker shall skip Where and start the agent in that terminal's folder. | Shot 735-04 |
| REQ-006 | WHEN Open Agent Here is chosen on a folder in the project panel, the picker shall skip Where and start the agent in that folder. | Shot 735-05 |
| REQ-007 | The `+` menus and Home shall offer New Agent…, and Home shall no longer say threads need a project folder. | Shot 735-01's rail and the review |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `agents.rs` (the Where step), `rail.rs` (New Agent…, Open Agent Here…),
  `home_page.rs`, the project panel touch with its ledger rows, the guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
