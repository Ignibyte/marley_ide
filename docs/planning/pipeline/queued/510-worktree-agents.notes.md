# Worktree agents: an agent on its own branch and worktree, nested under its project — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-510-worktree-agents.md
- **Pipeline spec:** 510-worktree-agents.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 5 of the list after the browser
  waves, first half: an agent in its own worktree on its own branch. Revised the same day in his
  answers to the Orca survey (`docs/orca_architecture/README.md`, open question 3): worktree agents
  are rows nested under their project in the rail, not projects of their own. The lead's brief
  names the create path's parts: Zed's worktree service with a small touch for a branch,
  `--no-track`, `branch.<b>.base`, the first prompt on the agent's argv, `.worktreeinclude`,
  Claude Code's `.claude/worktrees` kept out of the rail, the folder-trust prompt, and a port
  offset (report 02 §2.1 to §2.5 and §3 items 1, 6, 7; report 01 §3 item 6; report 06 §3 item 4).
- **Classification / tier:** feature, prong 2. Too big for one slice with everything the brief
  lists, so it splits (below): this slice is the worktree, the branch, the agent and the rows (M);
  slice 2 is the worktree's environment (S to M). Two small Zed touches, in `crates/git_ui_core`
  and `crates/git`; the rest in `marley_agent`, `marley_rail` and `marley_workbench`.
- **Recall (§18.3):**
  - L-claude-458-a-projects-folder-events-come-before-its-group-is-rekeyed-001: a workspace's
    group key changes before the `MultiWorkspace` rekeys its group, so the rail reads the groups
    in a `cx.defer_in`. A worktree workspace added in the background joins its group the same
    way; the rows are built from the deferred read.
  - PR-claude-a-path-hook-resolves-the-files-own-repo-001: linked worktrees are one repository;
    compare them by the common git directory, never by the checkout's own path. The rows key on
    the main checkout (`RepositorySnapshot::main_worktree_abs_path`, which reads the common dir).
  - #455 (`455-a-first-terminal.spec.md`): a folder opened fresh in the Marley layout gets a first
    terminal from `routing::seed_first_terminal`, decided by `Workspace::opened_from_saved_state`
    (a Zed touch of #455). D7 adds a skip for the agent's workspace.
  - The fork-port memory: `script/clippy` runs cargo-shear, so a dependency added to
    `marley_workbench` (`git_ui_core`) must be used in the same change.
  - `docs/marley_architecture/marley_workbench.md:65`: the rail lists only the groups that have an
    open workspace, and flattens every member's terminals under the group's row (`build_snapshot`).
  - Brain: no consultation run by this drafting agent (read-only brief); the Planner's
    `brain_ask` at promotion is still owed.
- **Discovery (checked in the tree):**
  - `crates/git_ui_core/src/worktree_service.rs`: `start_worktree_creations` (463) asks
    `CreateWorktreeTarget::Detached { base_sha }` at 504 for every create;
    `resolve_worktree_branch_target` (359) turns `NewWorktreeBranchTarget::ExistingBranch { name }`
    into the start point; `create_worktree_workspace` (736, background open) and
    `handle_create_worktree` (688) both reach `create_worktree_workspace_inner` (755), as does the
    fetch-failure retry at 269; `do_create_worktree` (967) records the worktree as Zed's
    (`record_created_worktree_for_repo`, 1071) and opens it (`open_worktree_workspace`, 1150), which
    adds it to the window in the background (`multi_workspace.add`, 1367), runs
    `run_create_worktree_tasks` (1374) and carries trust over (`maybe_propagate_worktree_trust`,
    632). `handle_switch_worktree` (897) opens an existing worktree.
  - `crates/git/src/repository.rs`: `CreateWorktreeTarget` (302) has `NewBranch { branch_name,
    base_sha }` (309); `create_worktree`'s `NewBranch` arm (2265 to 2274) builds
    `git worktree add -b <branch> -- <path> <start>` with no `--no-track`. `Worktree` (291) carries
    `path`, `ref_name`, `sha`, `is_main`; `branch_name` (335) strips `refs/heads/`.
  - `crates/project/src/git_store.rs`: `Repository::create_worktree` (9190) takes any target; the
    remote handler builds `NewBranch` at 3905 only when a client asked for one (the client side
    converts at 9213); `path_for_new_linked_worktree` (9144); `RepositorySnapshot` (597) with
    `branch` (615) and `linked_worktrees` (624), refreshed with `GitWorktreeListChanged` (12371);
    `main_worktree_abs_path` (6261), `is_linked_worktree` (6288), `linked_worktree_short_name`
    (10872); `checkout_branch_in_worktree` (9279), the two-step path D1 rejects.
  - `crates/zed_actions/src/lib.rs`: `NewWorktreeBranchTarget` (305), `CreateWorktree` (323),
    `SwitchWorktree` (334).
  - `crates/git_ui_core/src/worktree_names.rs:61`, `generate_worktree_name`;
    `crates/git_ui_core/src/created_worktrees.rs`, the record; `assets/settings/default.json:1857`,
    `"worktree_directory": "../worktrees"`.
  - `crates/workspace/src/tasks.rs:235`, `run_create_worktree_tasks`, which gives hook tasks
    `ZED_WORKTREE_ROOT` and `ZED_MAIN_GIT_WORKTREE`; `crates/task/src/task_template.rs:95`,
    `TaskHook::CreateWorktree`.
  - `crates/workspace/src/persistence.rs:327`, `read_serialized_multi_workspaces`: a window
    restores only its active workspace (with the group keys), which is why D8 reads git.
  - `crates/project/src/project.rs:6590`, `ProjectGroupKey`, keyed on the main worktree paths.
  - `crates/marley_workbench/src/rail.rs`: `build_snapshot` (1734) walks each group's members and
    puts every member's terminals under the group's row (1761 to 1794); `render_project_menu`
    (1113) and `agent_cli_entries` (1206) build the + menu; `new_agent` (619) and
    `activate_workspace` (512).
  - `crates/marley_workbench/src/agents.rs:188`, `start_cli`: a center terminal through the
    launcher's factory, the startup handshake, then `marley_agent::launch_input`.
  - `crates/marley_agent/src/marley_agent.rs:90`, `launch_input`, the program name and Enter;
    `AgentKind` (29) and `program` (46).
  - `crates/marley_workbench/src/routing.rs:65`, `seed_first_terminal`.
  - `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot` (30), `Selection` (182), `Row`
    (253), `walk` (347), `parent` (478), `window_row` (548), `switcher_rows` (574),
    `hidden_rows_need_the_user` (610).
  - `crates/sidebar/src/sidebar.rs:564`, `linked_worktree_path_lists_for_workspaces`, and 656 to
    667, how Zed's sidebar names a linked worktree.
  - `crates/agent_ui/src/thread_worktree_archive.rs:112`, `build_root_plan`: archiving a Zed
    agent thread may remove a Zed-created worktree (a risk below).
  - Orca (MIT, read): `src/main/git/worktree-add.ts:200` (`--no-track -b`) and `:231`
    (`persistWorktreeCreationBase`, which writes `branch.<b>.base`);
    `src/main/ipc/worktree-remote.ts:397` and `:442-446`; `src/shared/agent-scratch-worktrees.ts`
    (`.claude/worktrees`, `.gsd-workspaces`); `src/shared/tui-agent-config.ts:75-80,104-111,132-143,186-189`;
    `src/shared/tui-agent-startup.ts:101-167`; `src/shared/tui-agent-startup-shell.ts:172-216`.
- **Decisions:** D1 to D10 in the spec, and the split.

### The split
- **Slice 1, this ticket:** the worktree on its branch, the base, the agent with its prompt, the
  rows.
- **Slice 2, a ticket of its own (numbered when filed), "the worktree's environment":**
  - `.worktreeinclude` at the main checkout's root, read with the `ignore` crate's
    `gitignore::GitignoreBuilder` (in `Cargo.lock`, 0.4.24) as Claude Code reads it: `.gitignore`
    syntax, and only paths that also are gitignored (`git check-ignore`), copied from the main
    checkout into the new worktree before the agent starts, with a size cap and the skipped
    entries named in a toast (Orca: `src/main/git/worktree-include-file.ts`,
    `src/main/ipc/worktree-include-copy-budget.ts`; Orca supports literal paths only, Marley the
    whole syntax).
  - A port offset: each worktree Marley made gets a slot, and its terminals and tasks get
    `MARLEY_PORT_OFFSET` and `PORT` (3000 plus the offset); the seam is the terminal's
    environment, which today comes from `terminal.env` settings and the directory's environment
    (`crates/project/src/terminals.rs:284,312`), so slice 2 decides between a Marley hunk in
    `TerminalBuilder` (where the #474 nonce already goes) and the launch line.
  - `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH` for `create_worktree` hook tasks, beside Zed's
    `ZED_MAIN_GIT_WORKTREE`.

### Design
- **Approach.**
  - *Zed touch 1, `crates/git_ui_core/src/worktree_service.rs`:* `pub fn
    create_worktree_workspace_on_branch(workspace, action, branch_name: String, window,
    fallback_focused_dock, cx) -> Task<Result<CreatedWorktreeWorkspace>>`, the background open of
    `create_worktree_workspace` with `Some(branch_name)`; `create_worktree_workspace_inner`,
    `do_create_worktree` and `start_worktree_creations` gain `new_branch: Option<String>`; at 504 the
    target is `NewBranch { branch_name, base_sha: base_ref }` when set, else `Detached` as today.
    The three existing callers (269, 695, 743) pass `None`. Each hunk carries `// Marley:`.
  - *Zed touch 2, `crates/git/src/repository.rs`:* `--no-track` pushed before `-b` in the
    `NewBranch` arm (2269).
  - *`marley_agent`:* `launch_input_with_prompt(kind, prompt)`: the program, then the prompt as its
    argument per agent (Claude and Codex positional, Gemini `--prompt-interactive`, OpenCode
    `--prompt`), quoted by `quote_argument`, Orca's portable rule reimplemented in Rust (a comment
    names `src/shared/tui-agent-startup-shell.ts`); an empty prompt is `launch_input`.
  - *`marley_rail`:* `ProjectSnapshot.worktrees: Vec<WorktreeSnapshot { key (the worktree's path),
    name, branch, open, terminals, matched }>`; `Selection::Worktree(String)`; `Row::Worktree`; the
    walk puts the project header, the main checkout's terminals, each worktree row and its
    terminals, then the threads; `parent` of a worktree's terminal is its worktree row and of a
    worktree row its project; `cycle_row` passes over worktree rows; `switcher_rows` and the
    attention roll-up include worktree terminals; the filter matches a worktree by name or branch
    and shows the terminals under a matched worktree. `Focus.worktree` names the displayed
    workspace's worktree, so with no terminal focused the worktree row is selected, not the header.
  - *`marley_workbench/src/rail.rs`:* `build_snapshot` sorts each group's members into the main
    checkout (root equals the group key's path) and worktrees, and adds the repository's
    `linked_worktrees` that no member has open, minus the `.claude/worktrees/` ones (a member open
    on such a folder keeps its terminals under the project row). A `WorktreeEntry` per row holds
    the member workspace, if any, and the path. The + menu gains the New Agent in Worktree submenu.
    A click shows an open worktree's workspace, else calls `handle_switch_worktree` with the
    project's workspace. The rail subscribes to each listed project's `GitStore` and refreshes,
    deferred, on `RepositoryUpdated` with `GitWorktreeListChanged`, `HeadChanged` or
    `BranchListChanged`.
  - *`marley_workbench/src/worktree_agents.rs` (new module):* the prompt modal (`ModalView`, an
    auto-height `Editor` in a `MarleyWorktreePrompt` key context, `marley::StartWorktreeAgent` on
    Enter, `menu::Cancel` on Escape); on start: the base from the project's repository snapshot
    (the main checkout's `branch`, else its `is_main` entry in `linked_worktrees`, else the head
    commit), the worktree path marked for the seed skip, `create_worktree_workspace_on_branch` with
    `CreateWorktree { worktree_name: Some(name), branch_target: ExistingBranch { name: base } }`,
    then `git config branch.agent/<name>.base <base>` through `util::command::new_command` in the
    main checkout (a failure is shown and the agent still starts), then the agent's center
    terminal in the new workspace with `launch_input_with_prompt`, the handshake and write copied
    from `start_cli` (shared through a helper in `agents.rs`). Errors reach
    `detach_and_prompt_err`.
  - *`marley_workbench/src/routing.rs`:* `seed_first_terminal` skips a workspace whose first root
    is marked (a global set the create flow fills and the seed drains).
  - *`marley_workbench/keymap.json`:* `MarleyWorktreePrompt > Editor`: `enter`, `shift-enter`
    (`editor::Newline`), `escape`.
- **File manifest.** Zed crates: `crates/git_ui_core/src/worktree_service.rs`,
  `crates/git/src/repository.rs`. Marley crates: `crates/marley_agent/src/marley_agent.rs`,
  `crates/marley_rail/src/marley_rail.rs`, `crates/marley_workbench/src/rail.rs`,
  `crates/marley_workbench/src/agents.rs`, `crates/marley_workbench/src/worktree_agents.rs` (new),
  `crates/marley_workbench/src/routing.rs`, `crates/marley_workbench/src/marley_workbench.rs` (the
  module, the action, `init`), `crates/marley_workbench/keymap.json`,
  `crates/marley_workbench/Cargo.toml` (`git_ui_core`). Test phase: `script/e2e/510-worktree-agents.sh`.
- **Ledger rows (`docs/marley/zed-touchpoints.md`, before the edits):**
  - `crates/git_ui_core/src/worktree_service.rs`: the pub `create_worktree_workspace_on_branch`
    and the `new_branch` parameter through the three private functions, `NewBranch` when it is set
    (#510). Why: Marley's worktree agents need a branch, and Zed's service only makes detached
    heads. On merge: keep the function and the parameter; if upstream grows its own branch
    option, move Marley onto it and drop the row.
  - `crates/git/src/repository.rs`: `--no-track` in `create_worktree`'s `NewBranch` arm (#510).
    Why: a worktree agent's branch must not track its start point, or `git status` reports it
    behind before anything is pushed. On merge: keep the flag in the arm; drop the row if upstream
    adds it.

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | steps: click the project's + (coordinates measured on the first run), move to New Agent in Worktree, open its submenu | `510-01-menu` |
| REQ-002 | steps: Enter on Claude Code | `510-02-prompt` |
| REQ-003 | steps: after REQ-005's start, click the main checkout's terminal row and run `git worktree list; git branch -vv --list 'agent/*'` (no `[upstream]` on the new branch) | `510-04-branch`; the run log |
| REQ-004 | steps: in the same terminal, `git config --get-regexp 'branch\..*\.base'` | `510-04-branch` |
| REQ-005 | setup: the stand-in `claude` first on the PATH; steps: type `fix the "login" form's label`, Enter, settle; the stand-in prints one bracketed argument | `510-03-agent` |
| REQ-006 | setup: `git worktree add -q $E2E_WORK/manual -b manual`; the rail at the first shot (`manual`, not open), then with the new worktree's row | `510-01-menu`, `510-03-agent` |
| REQ-007 | the rail at `510-03-agent`: the Claude Code row under the new worktree's row, the main checkout's first terminal under the project row | `510-03-agent` |
| REQ-008 | setup: `git worktree add -q .claude/worktrees/scratch -b worktree-scratch` in the repository; the rail at the first shot | `510-01-menu` (no `scratch` row) |
| REQ-009 | steps: click the `manual` row | `510-05-opened` |
| REQ-010 | steps: focus the rail (`ctrl-alt-;`), select the agent's row, press Left | `510-06-keys` |
| REQ-011 | steps: `chmod a-w $E2E_WORK/worktrees` from the main terminal, New Agent in Worktree again; `chmod u+w` in teardown | `510-07-refused` |

What no scenario can reach: the real Claude Code's trust dialog in a new worktree (the scenario's
agent is a stand-in, and the real one would need Chad's login). P3 checks it once by hand: in a
worktree the scenario's flow made in a scratch repository, the real `claude '<prompt>'` shows the
trust dialog, and after Enter the prompt is the session's first message. The check sends no
prompt anywhere else, and its outcome goes in the Phase 3 entry.

### Risks
- Zed's thread archive removes worktrees Zed recorded as its own (`build_root_plan`, then
  `remove_root` with `--force`) when a Zed agent thread whose folder is that worktree is archived.
  A worktree agent's branch survives (only the worktree goes, and archive saves its state under
  `refs/archived-worktrees/`), but the worktree would vanish under a running terminal agent. P2
  checks whether archiving a thread in a Marley worktree reaches `remove_root`, and if so the
  create skips the record or the ticket files the fix; the notes say which.
- `git worktree list` can report a worktree whose folder was deleted (prunable). Its row opens
  nothing; the click's error says so. A `git worktree prune` stays Chad's call.
- A long prompt with newlines reaches bash and zsh as one quoted argument, but an interactive
  shell without bracketed paste shows continuation prompts while it is typed. The write is one
  string after the handshake; P2 decides whether to wrap it in a bracketed paste when the shell
  asked for one.
- The rail's rebuild runs on every terminal output. The rows must be built from the snapshot's
  `linked_worktrees` alone, with no git or filesystem call in the rebuild.
- A repository with many old worktrees (Zed's own layout collects them) gives the project many
  rows. Folding worktree rows is not in this slice; the project row still folds them all.

## Folded in from the Orca second pass (2026-09-26)
Smaller item 2 of `docs/planning/design-notes/orca-second-pass-2026-09-25.md`: a setup command
suggested for a repository that has none, for slice 2 (the worktree's environment).

- **What Orca does.** A sidebar card offers a setup command imported from another tool's file at
  the repository root, in this order: `conductor.json`, `.superset/config.json` (and
  `.superset/config.local.json`), `.cmux/cmux.json` or `cmux.json`, `.codex/environments/environment.toml`;
  or else, when `package.json` exists, the install command for the one lockfile present
  (`pnpm-lock.yaml` → `pnpm install`, `bun.lock` or `bun.lockb` → `bun install`, `yarn.lock` →
  `yarn install`, `package-lock.json` → `npm install`; lockfiles of two managers → no suggestion),
  so a new worktree has its dependencies before the agent's first command runs
  (`src/shared/setup-script-imports.ts`, `setup-script-package-manager-suggestion.ts`,
  `src/renderer/src/components/sidebar/SetupScriptPromptCard.tsx`; MIT, read).
- **For slice 2.** Zed already runs a repository's `create_worktree` task hooks after a create
  (`TaskHook::CreateWorktree`, `.zed/tasks.json`). When the repository has none, the prompt modal
  of this slice shows one more line under the branch line, `Setup: pnpm install (pnpm-lock.yaml
  found)`, with a checkbox that is off by default, since nothing a repository supplies runs until
  Chad says so; checked, the command runs in the worktree's terminal before the agent starts,
  and the choice is remembered per repository in `git config marley.worktreeSetup` (the command,
  or `none`), which every worktree reads and #511's `marley.merge` key sits beside. Orca's imported
  files are read for their command only, never run as they are; the JS lockfile table is Orca's,
  and other ecosystems (`uv.lock`, `Cargo.lock`) are added only when a real repository asks for
  them. A slice-2 acceptance row: WHEN the repository has no `create_worktree` task and one
  lockfile of one package manager, the modal shall offer that manager's install command, off by
  default; checked, it shall run in the worktree before the agent's command.
