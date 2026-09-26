---
pipeline_id: 6bac336b-5131-4521-bb8a-57d21fb2447a
ticket: docs/planning/tickets/open/TICKET-510-worktree-agents.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Worktree agents: an agent on its own branch and worktree, nested under its project"
type: feature
slice: prong 2, worktree agents, slice 1 of 2 (the worktree's environment is slice 2)
references: [docs/orca_architecture/02-worktrees-and-review.md, docs/orca_architecture/01-agents-and-sessions.md, docs/orca_architecture/06-cli-automations-skills.md, docs/planning/pipeline/completed/455-a-first-terminal.spec.md, docs/planning/pipeline/completed/458-rail-follows-folder-changes.spec.md]
---

## Title
A project's + in the rail starts an agent CLI in a new git worktree on its own branch, with its
first prompt, so several agents can work on one repository at once without touching each
other's files; the rail shows each worktree as a row nested under its project.

## Scope
### In
- **New Agent in Worktree** in a project's + menu, after the Agent CLIs section: a submenu with
  each agent CLI the search path holds (`agents::installed_clis`), none when none is installed.
- **The prompt.** Choosing an agent opens a small modal in the project's workspace: an editor for
  the first prompt (Enter starts, Shift-Enter adds a line, Escape cancels) over a line that names
  what will be made, `agent/<name> from main`. The name is generated when the modal opens, with
  Zed's `worktree_names::generate_worktree_name`, past the repository's existing worktree names
  and branches. An empty prompt starts the agent with no prompt.
- **The worktree,** made by Zed's worktree service on a new branch (D1, a Zed touch in
  `crates/git_ui_core`): at Zed's `git.worktree_directory` layout
  (`<parent>/worktrees/<name>/<project>` by default), on `agent/<name>` (D3), started from the
  main checkout's branch, or its commit when the main checkout is detached (D4), created with
  `--no-track` (D2, a Zed touch in `crates/git`). Zed keeps doing the rest: the created-worktree
  record, trust carried over from the main checkout, the `create_worktree` task hooks, the new
  workspace opened in the background as a member of the project's group.
- **The base,** written as `branch.agent/<name>.base <base>` with `git config` in the main
  checkout, so #511 and a later branch cleanup read it back without Marley state (D4).
- **The agent** in the worktree's one center terminal (D7): the routing's first-terminal seed
  skips a workspace Marley opens for an agent, and Marley opens the agent's terminal itself,
  writing the command after the shell's startup handshake, as `agents::start_cli` does, with the
  prompt on the command line (D5).
- **The rail's worktree rows** (D8, D9): under each project row, one row per linked worktree of
  the project's repository, named with Zed's `linked_worktree_short_name`, its branch (or short
  commit) on the second line; the terminals of a worktree whose workspace is open listed under
  its row; the main checkout's terminals directly under the project row, as today. No row for a
  worktree under the main checkout's `.claude/worktrees/`. A click on an open worktree's row
  shows its workspace; on one not open, Zed's `handle_switch_worktree` opens it. The keyboard
  (up, down, left to climb, Enter), the filter (a worktree matches by name or branch) and the
  switcher cover the new rows. The rail refreshes on the repository's worktree, head and branch
  events.
- **Errors:** a create that fails says why in a prompt, starts no agent and leaves no row.

### Out (explicitly deferred)
- **Slice 2, its own ticket (the worktree's environment):** copying the gitignored files a
  `.worktreeinclude` names, a port offset per worktree in its terminals and tasks
  (`MARLEY_PORT_OFFSET`, `PORT`), and `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH` for setup tasks
  (report 02 §2.2 to §2.4, §3 items 6 and 7). The notes carry its outline.
- Review, merge and removal: #511 and its second slice.
- Writing trust into Claude Code's or Codex's own configuration before a launch (D6).
- An editable branch name, a base other than the main checkout's branch, and a branch renamed
  from the first prompt (Orca's first-work rename, report 02 §2.5).
- Zed agent threads nested under their worktree: they stay under the project row.
- A worktree agent from the New Agent picker (`ctrl-alt-n`) or from an MCP tool
  (`worktree_create`, report 06 §3 item 4).
- The setting for agents' permission-bypass flags (Chad's decision of 2026-09-25, its own ticket).
- Remote (SSH) projects: the entry is not offered there.

## Reference (§20)
Upstream Zed is the reference and most of the implementation: its worktree service
(`git_ui_core::worktree_service`) names, places and creates the worktree, records that Zed made
it, carries trust over and runs the `create_worktree` task hooks, and its Threads Sidebar groups
workspaces by `ProjectGroupKey` and names a linked worktree with
`project::linked_worktree_short_name`. Marley keeps all of that and adds a branch. Orca's worktree
create (report 02 §2.1) and its sidebar of projects with worktrees under them (§2.5) are the
behavior the rows follow, and Claude Code's published worktree docs define the
`.claude/worktrees/` folders the rail leaves out. Warp: N/A. This is project and workspace
behavior, and the Warp once-over (`docs/planning/design-notes/warp-once-over-2026-09-25.md`)
ruled worktrees out as planned here.

### Prior art
- **Behavior maps and reports.** Orca report 02: §2.1 (`git worktree add --no-track -b`, then
  `git config --local branch.<b>.base`, `src/main/git/worktree-add.ts:200,231`; the agent's
  prompt on its argv, `src/main/ipc/worktree-remote.ts:397`; trust written ahead for Cursor,
  Copilot and Codex, `:442-446`), §2.5 (projects as top rows with worktrees under them; Claude
  Code's `.claude/worktrees/` hidden, `src/shared/agent-scratch-worktrees.ts`), §3 item 1.
  Report 01 §3 item 6 and Orca's launch table, `src/shared/tui-agent-config.ts` (Claude and Codex
  take the prompt as an argument, OpenCode `--prompt`, Gemini `--prompt-interactive`),
  `src/shared/tui-agent-startup.ts:101-167`, and `src/shared/tui-agent-startup-shell.ts:172-216`:
  fish treats `\\` and `\'` as escapes inside single quotes, so the sh `'\''` idiom breaks there,
  and Orca writes an apostrophe as `"'"` and a backslash as `"\\"` between single-quoted runs,
  which every Unix shell reads the same. Report 06 §3 item 4 (the same path later behind an MCP
  tool).
- **Published material.** Claude Code's worktree page (code.claude.com/docs/en/worktrees):
  worktrees under `.claude/worktrees/<name>/` on `worktree-<name>`, `.worktreeinclude` in
  `.gitignore` syntax copying only ignored files (slice 2), and "Interactive runs require workspace
  trust: if you haven't run Claude in the directory before, run `claude` once there to accept the
  trust dialog". Its security page: "trust acceptance is saved per directory". `git-worktree(1)`:
  a new branch tracks its start point by default when that is a remote-tracking branch;
  `--no-track` turns it off.
- **Code we already ship.** Zed's `worktree_service.rs` (`create_worktree_workspace`, which
  always asks `CreateWorktreeTarget::Detached`, line 504), `CreateWorktreeTarget::NewBranch` in
  `crates/git/src/repository.rs`, built outside tests only by the remote-project handler
  (`git_store.rs:3905`), `created_worktrees.rs`, `worktree_names.rs`, `handle_switch_worktree`,
  `RepositorySnapshot::linked_worktrees` with `RepositoryEvent::GitWorktreeListChanged`, and
  `linked_worktree_short_name`, which Zed's sidebar already uses. Marley's own
  `agents::start_cli`, `marley_agent::launch_input`, `routing::seed_first_terminal` and the rail's
  `build_snapshot`. The sweep's win: Zed already owns naming, layout, creation, rollback, trust and
  hooks, so the Zed side of this ticket is one parameter and one flag.

## UI proof
UI-AFFECTING. `script/e2e/510-worktree-agents.sh` (`compositor sway`: it clicks the rail). Setup:
a scratch repository on `main` with a commit; a stand-in `claude` first on the PATH that prints
each argument it got in brackets and then runs as `claude` (`exec -a claude sleep`), so the rail
takes it for Claude Code; a Claude Code-style worktree at `.claude/worktrees/scratch` on
`worktree-scratch`; a worktree made by hand at `$E2E_WORK/manual` on `manual`; a HOME whose
`.bashrc` sets a plain prompt. Shots: `510-01-menu` (the + menu with New Agent in Worktree and
Claude Code under it; the rail's `manual` row, not open, and no `scratch` row), `510-02-prompt`
(the modal naming `agent/<name> from main`), `510-03-agent` (after a prompt with an apostrophe and
quotes: the new worktree's row under the project with the Claude Code row under it, the stand-in's
one bracketed argument in the terminal), `510-04-branch` (the main checkout's terminal:
`git branch -vv --list 'agent/*'` with no upstream, `git config --get-regexp 'branch\..*\.base'`
naming `main`), `510-05-opened` (a click on `manual`: its workspace open, its row selected, its
terminal under it), `510-06-keys` (Left from the agent's row selects the worktree row), and
`510-07-refused` (the worktrees folder made read-only, a second New Agent in Worktree: the reason
shown, no new row).

## Locked-In Decisions
- D1 — Zed's worktree service makes the worktree, with one added parameter, the branch: a new pub
  `create_worktree_workspace_on_branch` in `worktree_service.rs` threads it through
  `create_worktree_workspace_inner`, `do_create_worktree` and `start_worktree_creations`, which ask
  `CreateWorktreeTarget::NewBranch` when it is given and `Detached` as before when not; every
  existing caller passes none. Rejected: Zed's detached create followed by
  `Repository::checkout_branch_in_worktree`, the two steps thread archive's restore takes, since the
  `create_worktree` hooks would run on a detached HEAD and a failed checkout would leave a detached
  worktree behind.
- D2 — `--no-track` in the `NewBranch` arm of Zed's `git worktree add`: the branch never gets an
  upstream, whatever its start point, so `git status` in the worktree never says "behind
  origin/main" before anything is pushed. Outside tests only Marley's new path and the
  remote-project handler build that arm, and no Zed client asks the handler for it.
- D3 — The branch is `agent/<name>`, `<name>` the worktree's generated name: `git branch` shows
  whose work it is, and it never collides with Chad's own `marley/…` branches.
- D4 — The base is the main checkout's branch when the worktree is made (its commit when it is
  detached), passed as the start point and written as `branch.agent/<name>.base` in the
  repository's config, which every worktree of it reads. Marley keeps no worktree state of its own.
- D5 — The first prompt goes on the agent's command line, never typed into its TUI: `claude <p>`,
  `codex <p>`, `gemini --prompt-interactive <p>`, `opencode --prompt <p>` (Orca's table), quoted by
  Orca's portable rule, so bash, zsh and fish read the same argument. The quoting lives in
  `marley_agent`, pure, beside `launch_input`.
- D6 — Marley writes nothing into another tool's configuration to trust the new folder. Claude
  Code saves trust per directory, so a new worktree shows its trust dialog; the prompt sits on the
  command line, where the dialog cannot take it, and runs once Chad answers. The rail shows the
  agent waiting meanwhile.
- D7 — The agent's terminal is the worktree's only one: Marley marks the worktree's path before the
  create, the Marley layout's first-terminal seed skips a workspace so marked, and Marley opens the
  agent's center terminal itself. Rejected: reusing the seeded terminal, which the seed opens
  asynchronously, so the agent's start would race it.
- D8 — The rows come from git, not from what the window holds open: every linked worktree of the
  project's repository (`RepositorySnapshot::linked_worktrees`) except the main checkout and those
  under the main checkout's `.claude/worktrees/`, so a worktree keeps its row after a restart,
  which restores only each window's active workspace. A workspace open on a `.claude/worktrees/`
  folder lists its terminals under the project row, as every member does today.
- D9 — A click on an open worktree's row shows its workspace, as a project header does; on a row
  whose workspace is not open it runs Zed's `handle_switch_worktree`, which opens the worktree in
  the window.
- D10 — The entry is offered for a local project with a git repository; the rail builds the rows
  from pure data in `marley_rail`, and the rebuild runs no git.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens a git project's + menu in the rail, the menu shall list New Agent in Worktree with each installed agent CLI under it. | Shot `510-01-menu` |
| REQ-002 | WHEN the user chooses an agent CLI under New Agent in Worktree, the system shall open a prompt for the agent's first prompt that names the branch and the base it will create. | Shot `510-02-prompt` |
| REQ-003 | WHEN the user submits the prompt, the system shall create a linked worktree under Zed's `git.worktree_directory` on a new branch `agent/<name>` that starts at the main checkout's branch and has no upstream. | Shot `510-04-branch`; the run log's `git worktree list` |
| REQ-004 | WHEN the worktree is created, the system shall write the main checkout's branch as `branch.agent/<name>.base` in the repository's git config. | Shot `510-04-branch` |
| REQ-005 | WHEN the worktree is created, the system shall start the chosen agent CLI in the worktree's only center terminal with the first prompt, quotes and apostrophe included, as a single argument on its command line. | Shot `510-03-agent` |
| REQ-006 | WHILE a project has linked worktrees, the rail shall show one row for each under the project's row, with the worktree's name and its branch. | Shots `510-01-menu`, `510-03-agent` |
| REQ-007 | WHILE a worktree's workspace is open, the rail shall list that workspace's terminals under the worktree's row, and the main checkout's terminals under the project's row. | Shot `510-03-agent` |
| REQ-008 | WHERE a linked worktree lies under the main checkout's `.claude/worktrees/`, the rail shall show no row for it. | Shot `510-01-menu` |
| REQ-009 | WHEN the user clicks the row of a worktree whose workspace is not open, the system shall open that worktree's workspace in the window and show it. | Shot `510-05-opened` |
| REQ-010 | WHILE the rail holds focus, Left on a terminal row under a worktree shall move the selection to the worktree's row. | Shot `510-06-keys` |
| REQ-011 | IF the worktree cannot be created, THEN the system shall show the reason, start no agent and add no row. | Shot `510-07-refused` |

## Phase Plan
- **P1 Plan** — this spec, and the design and the E2E plan in the notes.
- **P2 Code** — the two Zed touches with their rows in `docs/marley/zed-touchpoints.md` first;
  `marley_agent`'s prompt launch and quoting; `marley_rail`'s worktree rows; the workbench's menu
  entry, modal, create flow, base write, agent start and rows; the routing's seed skip; fmt and
  clippy clean; a review of the diff (§18.1).
- **P3 Test** — write and run `script/e2e/510-worktree-agents.sh`, read every shot; once, by hand,
  the real Claude Code in a new worktree to see the trust dialog keep the prompt (notes);
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_rail.md` and `marley_workbench.md`,
  the plan's prong 2, the touchpoint rows checked against what shipped, ledger capture, close,
  archive, commit; file slice 2's ticket.
