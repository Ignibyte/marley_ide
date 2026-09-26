---
pipeline_id: 3df3b046-8a7a-4e68-b152-db9a25aadf76
ticket: docs/planning/tickets/open/TICKET-520-terminal-identity.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Each terminal knows its id, and Marley's tools know their caller"
type: feature
slice: prong 2, C0 follow-on (the caller behind a tool call); prong 3 (browser tools scoped to the caller's project)
references: [docs/orca_architecture/README.md, docs/orca_architecture/06-cli-automations-skills.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/completed/501-zeds-agents-drive-the-browser.spec.md]
---

## Title
Every local Marley terminal starts with `MARLEY_TERMINAL_ID`, a UUID of its own that survives
session restore, and `MARLEY_PROJECT`, its project's folder; values it inherited are replaced.
The plugin's bridge forwards both, with its own working directory, as headers on each request;
`marley_mcp` hands the caller to the app. `terminal_list` then marks the caller's own terminal,
`terminal_blocks` and `terminal_read` default to it, and a browser tool that names no tab acts in
the caller's project instead of on whichever tab the user focused last.

## Scope
### In
- **The variables.** `MARLEY_TERMINAL_ID` (a v4 UUID) and `MARLEY_PROJECT` (the absolute path of
  the project's first folder, `Project::first_project_directory`) in each local interactive
  terminal. Whatever the environment held under those names (Marley's own environment, the
  directory environment Zed resolves, a split's source) is replaced; a task terminal and a remote
  one get neither.
  - `TerminalBuilder::new` (Zed's `terminal` crate), beside #474's nonce: it removes both
    variables from a task's or a remote terminal's environment; for a local interactive one it
    sets `MARLEY_TERMINAL_ID` to the id a restore handed it or to a new one, and the `Terminal`
    keeps it (`marley_terminal_id()`).
  - `Project::create_terminal_shell_internal` (Zed's `project` crate) sets `MARLEY_PROJECT` for a
    local terminal and passes a restored id to the builder; a new
    `create_terminal_shell_restoring(cwd, terminal_id)` is what the restore calls.
  - `marley_terminal::identity` (Marley): the names, `new_terminal_id`, and the shape check.
- **Restore.** A Marley table keeps each terminal item's id (`MarleyTerminalIdsDb`, a database
  domain in `marley_workbench` like #494's `MarleyBrowserTabsDb`). `TerminalView`'s serialize,
  deserialize and cleanup reach it through a hook global `marley_workbench` sets, as
  `MarleyTerminalFooter` (#477) is set.
- **The bridge** sends `Marley-Terminal` (a UUID), `Marley-Project` and `Marley-Cwd` (absolute
  paths, at most 4,096 bytes each) with every request, each only when its value is well formed.
- **`marley_mcp`.** The transport reads the three headers into a `Caller` and `AppCall` carries it
  (`AppCall::caller`). A malformed value is dropped, not refused.
- **The terminal tools.** `terminal_list` gives each terminal's `terminal_id` and marks the
  caller's own with `"self": true`. `terminal_blocks` and `terminal_read` with no `terminal` read
  the caller's; with no caller either, they refuse with the reason and the next step
  (`terminal_list`).
- **The browser tools.** The caller's project is its terminal's project group when the id
  resolves, else the group with a workspace whose folders hold `Marley-Project`, else the one
  holding `Marley-Cwd` (the longest folder wins). The workspaces' own folders are matched, not
  the group's key, so a linked worktree (#510) finds its group. A tool that names no `tab` acts
  on the tab the user focused last in that project; `browser_navigate` with none opens a new tab
  there when the project has no tab. `browser_tabs` gives each tab's project and marks the tab a
  call with no `tab` would act on for this caller.
- **Zed's own agents.** The context server Marley registers (#501) sets both variables empty in
  the bridge's environment, so a Marley started from a Marley terminal passes its parent's identity
  to no one. Zed starts a local project's context servers in the project's folder, so an Agent
  Panel agent is placed by `Marley-Cwd`.
- `marley_mcp`'s tool descriptions say what a call with no `terminal` or `tab` acts on.

### Out (explicitly deferred)
- `FORCE_HYPERLINK=1` (Orca sets it for OSC 8 links; #503's business).
- Remote terminals: they get no id, and their agents cannot reach the loopback server anyway.
- A thread id for Agent Panel callers; refusing a stale id with a next step (Orca's remint): a
  Marley terminal ends with Marley, so its id cannot outlive the app.
- #519's `fleet_snapshot` seats keep `terminal_list`'s id; moving them to the UUID is a follow-up.
- Codex's, Gemini's and OpenCode's MCP setup, which would carry the same bridge.
- #507's browser context per project: this ticket only chooses the tab.

## Reference (§20)
- **Orca:** every PTY starts with its identity in the environment (`ORCA_TERMINAL_HANDLE`,
  `ORCA_PANE_KEY`, `ORCA_WORKTREE_ID`), inherited identity is scrubbed, and a command's
  "current" worktree is the one whose path holds its working directory (report 06 §2.5 and
  item 1). Browser commands default to that worktree's active tab (report 03 §2.11 and item 4).
  Marley does the same over MCP, with the bridge as the carrier.
- **Upstream Zed:** the terminal's environment as Zed builds it (`insert_zed_terminal_env`,
  `TerminalBuilder::new`), terminal items saved and restored per workspace (`TerminalView`'s
  serialize and deserialize), and context servers started in the project's folder. All kept;
  Marley adds two variables and a table beside them.
- **Warp:** N/A. This is plumbing between an agent and Marley's MCP server, with no Warp
  behavior to match.

### Prior art
- **Reports.** Report 06 §2.5 names Orca's spawn environment
  (`src/main/providers/local-pty-spawn-environment.ts`, `src/main/ipc/pty/provider/local-configure.ts`,
  `src/main/ipc/pty/host-env/assembly.ts`), its scrub of inherited identity
  (`removeUnspecifiedPaneIdentityEnv`), its "who am I" order
  (`src/cli/handlers/orchestration/terminal-identity.ts`) and its cwd rule (`src/cli/selectors.ts`);
  §2.16's table puts "say who is calling" in both places, an environment variable forwarded by
  the bridge. Report 03 §2.11 gives Orca's browser targeting (`getBrowserCommandTarget` in
  `src/cli/selectors.ts`).
- **Published material.** MCP's Streamable HTTP transport, whose own headers (`Mcp-Session-Id`,
  `MCP-Protocol-Version`) the bridge already sends; other request headers are the client's. The
  installed Claude Code 2.1.283, its bundle read and not run: a stdio MCP server starts with
  Claude Code's own environment plus the server's `env`, unless `CLAUDE_CODE_MCP_ALLOWLIST_ENV`
  is set, so the bridge sees the terminal's variables.
- **Code we already ship.** #474's nonce is the pattern: a per-terminal value set in
  `TerminalBuilder::new` for local terminals only. `Terminal::clone_builder` rebuilds a split
  from its source's saved environment, so the builder, not the project, must mint the id.
  `Project::first_project_directory` gives the folder. #494's Browser tab table and L-claude-494
  (item ids change at each launch; save on join, clean up the rest) give the restore's shape;
  #477's `MarleyTerminalFooter` gives the hook's. Zed's `ContextServerStore` starts a local
  project's stdio servers in its root (`crates/project/src/context_server_store.rs:1059`). The
  `uuid` crate is a workspace dependency. #491's stand-in client finds "its own" terminal by
  guessing from the running command, the gap this ticket closes.

## UI proof
UI-AFFECTING: what agents see through the tools, shown in terminals and Browser tabs.
`script/e2e/520-terminal-identity.sh` (`compositor sway`: a split, the rail's rows and its + menu
are clicked). Setup exports `MARLEY_TERMINAL_ID=inherited` and `MARLEY_PROJECT=/nowhere` before
the launch, so Marley inherits them; builds scratch repositories A (opened) and B (opened in the
steps through Zed's Open with its own path prompt, `use_system_path_prompts` off in the run's
settings), B holding a `.zed/tasks.json` task that prints both variables; a HOME whose `.bashrc`
defines `mcp` (#491's stand-in client, with `list`, `blocks`, `tabs` and `navigate`) and `ids`
(prints both variables and appends them to a log); and an offline Chromium serving two local
pages (`browser-fixture.sh`). Shots:
- `520-01-identity`: `ids` in A's terminal prints a UUID and A's folder; after a split, `ids` in
  the new terminal prints another UUID.
- `520-02-self`: `mcp list` in A's first terminal: its row reads `self: true`, the split's does
  not, and each row has a `terminal_id`.
- `520-03-own-blocks`: `mcp blocks` with no terminal named lists that terminal's commands.
- `520-04-task`: B's task prints both variables unset.
- `520-05-scoped-tab`: B gets a Browser tab from the rail's + (so the tab the user focused last is
  B's); in A's terminal, `mcp navigate <page>` opens the page in a new tab in A, and B's tab keeps
  its page.
- `520-06-agent-panel`: #501's stand-in ACP agent, started in B, is prompted to open the other
  page: it opens in B.
- `520-07-restored`: after `quit_marley` and `launch_marley`, `ids` in A's restored terminal
  prints the id it had before; the run log compares the two log lines.
A client run from the harness with an empty environment calls `terminal_list` and
`browser_navigate`: no row is `self`, and the page loads in the tab the user focused last, in
whichever project, as before this ticket (the run log).

## Locked-In Decisions
- D1: The id is a UUID that Marley mints, never the gpui entity id, which changes at every
  launch (L-claude-494) and which the terminal's programs never see.
- D2: The builder mints it. Every PTY's environment passes through `TerminalBuilder::new` last,
  a split rebuilds from its source's saved environment, and Marley's own environment can carry a
  parent terminal's id into the directory environment Zed resolves. So the builder removes what it
  was handed and sets its own, beside #474's nonce. `MARLEY_PROJECT` comes from the project, the
  one place that knows the folder; a split keeps its source's, the same project.
- D3: A restored id reaches the builder under a private key that the builder removes before it
  saves the environment for splits, so a split of a restored terminal still gets a new id.
- D4: The id's row lives in a Marley table reached through a hook, not in a new column of Zed's
  `terminals` table. That table's migrations are a list upstream appends to; a Marley migration
  in it would collide with upstream's next one at a merge.
- D5: The bridge sends the headers on every request. The transport answers each request on its
  own, and three short headers cost nothing.
- D6: The caller's project comes from its terminal, else `Marley-Project`, else `Marley-Cwd`,
  matched against each group's workspaces' own folders with the longest winning (Orca's cwd
  rule, report 06 §2.5). A group's key holds main worktree paths only
  (`ProjectGroupKey::from_project`), which a linked worktree's folder is not under.
- D7: A caller with no project keeps today's behavior, so a client outside Marley's terminals
  loses nothing.
- D8: The context server Marley registers (#501) blanks both variables. Zed's agents are placed
  by their bridge's folder, the project root Zed starts their server in.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley opens a local terminal, its environment shall hold its own `MARLEY_TERMINAL_ID`, a UUID, and `MARLEY_PROJECT`, its project's folder, whatever values Marley inherited. | Shot `520-01-identity` |
| REQ-002 | WHEN a terminal is split, the new terminal shall get a new `MARLEY_TERMINAL_ID`. | Shot `520-01-identity` |
| REQ-003 | WHEN a task runs, its environment shall hold neither variable. | Shot `520-04-task` |
| REQ-004 | WHEN Marley restores a terminal at launch, the terminal shall keep its `MARLEY_TERMINAL_ID`. | Shot `520-07-restored`; the run log's comparison |
| REQ-005 | WHEN an agent in a Marley terminal calls `terminal_list` through the plugin's bridge, the answer shall mark its own terminal `self` and give each terminal's `terminal_id`. | Shot `520-02-self` |
| REQ-006 | WHEN `terminal_blocks` or `terminal_read` names no terminal, it shall read the caller's own. | Shot `520-03-own-blocks` |
| REQ-007 | WHEN a browser tool names no tab, it shall act in the caller's project: on its tab the user focused last, or for `browser_navigate` on a new tab there when it has none. | Shot `520-05-scoped-tab` |
| REQ-008 | WHEN an Agent Panel agent calls a browser tool with no tab, it shall act in its thread's project. | Shot `520-06-agent-panel`; the stand-in's log |
| REQ-009 | WHEN a caller has no terminal and no project, the tools shall act as they did before this ticket. | The run log: the harness-side client |
| REQ-010 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, re-verify the seams (#516 is editing `mcp.rs`,
  `browser_tools.rs` and `marley_mcp` tonight), and read `delete_unloaded_items` and the
  terminal view's `cleanup` before choosing the hook's shape.
- **P2 Code:** the ledger rows first (`crates/terminal/src/terminal.rs`,
  `crates/project/src/terminals.rs`, `crates/project/Cargo.toml`,
  `crates/terminal_view/src/terminal_view.rs`); the variables and the builder; the table and the
  hook; the bridge's headers and the `Caller`; the terminal and browser tools; the context
  server's blank variables. fmt and clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario, read every shot, `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
