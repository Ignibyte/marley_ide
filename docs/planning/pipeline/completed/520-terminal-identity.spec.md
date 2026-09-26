---
pipeline_id: 3df3b046-8a7a-4e68-b152-db9a25aadf76
ticket: docs/planning/tickets/open/TICKET-520-terminal-identity.md
status: Phase 4 — Complete PASS
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
- **The bridge** sends `Marley-Terminal` (a UUID), `Marley-Project` and `Marley-Cwd` (absolute
  paths, at most 4,096 bytes each) with every request, each only when its value is well formed.
- **`marley_mcp`.** The transport reads the three headers into a `Caller` and `AppCall` carries it
  (`AppCall::caller`). A malformed value is dropped, not refused.
- **The terminal tools.** `terminal_list` gives each terminal's `terminal_id` and marks the
  caller's own with `"self": true`. `terminal_blocks` and `terminal_read` with no `terminal` read
  the caller's; with no caller either, they refuse with the reason and the next step
  (`terminal_list`).
- **Zed's own agents.** The context server Marley registers (#501) sets both variables empty in
  the bridge's environment, so a Marley started from a Marley terminal passes its parent's identity
  to no one.
- `marley_mcp`'s tool descriptions say what a call with no `terminal` acts on.

### Out (explicitly deferred)
- **Cut at promotion (2026-09-26):** the browser tools acting in the caller's project (a tab
  named by none, `browser_navigate`'s new tab, `browser_tabs`' `project` and `default`, an Agent
  Panel agent placed by `Marley-Cwd`; the queued REQ-007 and REQ-008) are TICKET-574, which
  reads this slice's `Caller`; the id surviving a restore (the table and the terminal view's
  hook; REQ-004) is TICKET-575. Until then an id lasts one launch.
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
UI-AFFECTING: what agents see through the tools, shown in terminals.
`script/e2e/520-terminal-identity.sh` (`compositor sway`: Chad's own Marley is open, and the
runner refuses Hyprland beside it; keys only). Setup exports `MARLEY_TERMINAL_ID=inherited` and
`MARLEY_PROJECT=/nowhere` before the launch, so Marley inherits them; builds a scratch repository
holding a `.zed/tasks.json` task that prints both variables; and a HOME whose `.bashrc` defines
`ids` (prints both variables). The stand-in agent (`browser-fixture.sh`'s `mcp_agent`, through the
plugin's bridge) runs in a terminal, so its requests carry that terminal's headers. Shots:
- `520-01-identity`: `ids` prints a UUID and the repository's folder; after a split (`pane: split
  right`), `ids` in the new terminal prints another UUID.
- `520-02-self`: `mcp_agent terminals` in the first terminal: its row reads `self`, the split's
  does not, and each row has a `terminal_id`.
- `520-03-own-blocks`: `mcp_agent blocks-here`, `terminal_blocks` with no terminal named, lists
  that terminal's own commands.
- `520-04-task`: the task, run from the palette, prints both variables unset.
The harness-side client (the runner's shell, no terminal of Marley's) calls `terminal_list`: no
row is `self`, and `terminal_blocks` with no terminal refuses with its next step (the run log).

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
| REQ-003 | WHEN a task runs, both variables shall be empty in its environment, naming no terminal and no project. | Shot `520-04-task` |
| REQ-005 | WHEN an agent in a Marley terminal calls `terminal_list` through the plugin's bridge, the answer shall mark its own terminal `self` and give each terminal's `terminal_id`. | Shot `520-02-self` |
| REQ-006 | WHEN `terminal_blocks` or `terminal_read` names no terminal, it shall read the caller's own. | Shot `520-03-own-blocks` |
| REQ-009 | WHEN a caller has no terminal, the tools shall act as they did before this ticket. | The run log: the harness-side client |
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
