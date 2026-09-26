# Each terminal knows its id, and Marley's tools know their caller — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-520-terminal-identity.md
- **Pipeline spec:** 520-terminal-identity.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and
  bring it in here". The survey's second item: terminal identity in the environment, forwarded by
  the bridge, so every tool is scoped to its caller (`docs/orca_architecture/README.md` item 2;
  report 06 §2.5 and item 1; report 03 item 4). Queued overnight by the spec drafters.
- **Classification / tier:** feature, prong 2 (C0 follow-on) with prong 3's browser tools. Four
  Zed files change, each by a small additive hunk; the rest is Marley's. Size M.
- **Recall (§18.3):**
  - AD-claude-474: the nonce is set per local terminal in `TerminalBuilder::new` and kept from
    remote hosts; the id follows the same pattern and the same place.
  - L-claude-494: an item's id is new at each launch; Zed deserializes under the saved id, saves
    again under the new one, then cleans up by the loaded items' ids. A Marley table keyed by
    item id stays right only if it saves on join and deletes the rest, as #494's Browser tab
    table does.
  - AD-claude-477: code Marley runs inside Zed's terminal view goes through a hook global Marley's
    workbench sets (`MarleyTerminalFooter`); the restore hook is the same kind.
  - AD-claude-491, L-claude-491 and AD-claude-501: the server runs in the app behind the stdio
    bridge; Zed's agents reach it through the context server `marley`, which Zed runs per
    project; L-claude-501 says every Agent Panel agent takes it.
  - PR-claude-a-marley-crate-writes-from-the-contract-not-the-gpl-body-001: the Marley table
    follows `MarleyBrowserTabsDb`'s shape without copying Zed's terminal queries.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery:** each seam opened and checked on 2026-09-25; line numbers are at commit
  `520a6e22a7`. #516, active the same night, edits `mcp.rs`, `browser_tools.rs` and
  `marley_mcp`, so promotion re-reads those.
  - `crates/terminal/src/terminal.rs:1153` (`TerminalBuilder::new`, taking `env` by value),
    `:1207` (`insert_zed_terminal_env`), `:1209-1219` (#474's nonce, local terminals only),
    `:1221` (the shell integration, local interactive shells only), `:1430-1439` (the
    `CopyTemplate` saves `env` after those changes), `:3374` (`clone_builder` rebuilds a split
    from `self.template.env`), `:716-725` (`insert_zed_terminal_env`: `TERM_PROGRAM=zed`, `ZED_TERM`).
  - `crates/project/src/terminals.rs:54` (`first_project_directory`), `:64`
    (`create_terminal_task`, its env from the resolved directory environment, `:136`), `:284`
    (`create_terminal_shell`), `:312` (`create_terminal_shell_internal`: the resolved directory
    environment, then `env.extend(settings.env)` at `:378`, then `TerminalBuilder::new` at
    `:409`), `:453` (`clone_terminal`), `:517` (`exec_in_shell`, not a terminal). `project` does
    not depend on `marley_terminal` today; `terminal` and `terminal_view` do.
  - `crates/terminal_view/src/terminal_view.rs:1923` (`cleanup`, `delete_unloaded_items` over
    Zed's `terminals` table), `:1933` (`serialize`: the working directory and the custom title,
    only while `needs_serialize`), `:1970` (`deserialize`: reads both, then
    `project.create_terminal_shell(cwd)`), `:1158-1166` (the first event that reports a working
    directory sets `needs_serialize`, so a restored terminal saves again under its new item id),
    `:130-145` (`MarleyFooterContext`, `MarleyTerminalFooter`), `:150` (`MarleyTerminalSuggestion`).
  - `crates/terminal_view/src/persistence.rs:416-456` (the `terminals` migrations, a list
    upstream appends to: `working_directory_path` and `custom_title` came that way).
  - `crates/marley_workbench/src/browser.rs:4841-4935` (`MarleyBrowserTabsDb`: its own domain,
    `static_connection!(…, [WorkspaceDb])`, `delete_unloaded_items` in `cleanup`).
  - `crates/marley_terminal/src/shell_integration.rs:44-50` (`NONCE_VARIABLE`, `new_nonce`); the
    crate's dependencies are `alacritty_terminal`, `log`, `marley_dcs` and `rand`.
  - The bridge: `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge:113-127`
    (`request` builds the headers), `:220-233` (`forward`).
  - `crates/marley_mcp/src/transport.rs:381-388` (`HttpRequest`: method, origin, bearer, session,
    body), `:399-449` (`read_http_request` keeps only those headers), `:280-293` (`ask_app` builds
    the `AppCall`); `crates/marley_mcp/src/marley_mcp.rs:130-165` (`AppCall`: tool, arguments,
    the answer's sender).
  - `crates/marley_workbench/src/mcp.rs:118-168` (the context server `marley`, its only env
    `MARLEY_MCP_ENDPOINT`), `:219-231` (`answer`), `:235-263` (`terminals`), `:266-280`
    (`terminal_with_id`, `terminal_argument`), `:282-317` (`terminal_list`: the view's entity id,
    title, project, cwd, running command, block count).
  - `crates/marley_workbench/src/browser_tools.rs:106-129` (`page_of`: the named tab, else
    `hub.focused()`), `:423-462` (`navigate`: a new tab when asked or when the browser has none);
    `crates/marley_workbench/src/browser.rs:520-525` (`BrowserHub::focused`: the page the user
    focused last, else the newest), `:527` (`set_focused`, called from `:3305`), `:535`
    (`tabs`), `:2798-2874` (`place_tab`: beside the focused tab, else in the active workspace),
    `:3005` (`BrowserView.workspace`).
  - `crates/project/src/context_server_store.rs:1059` (a local project's stdio context server
    starts in `root_path`), `:1870-1880` (`working_directory_for`).
  - `crates/project/src/project.rs:6579` (`Project::project_group_key`), `:6590-6611`
    (`ProjectGroupKey` keeps the main worktree paths only, `from_project`), `:6623`
    (`path_list`): a linked worktree's folder is not under its group's key, so the fallback
    matches the group's workspaces' own folders (`Workspace::root_paths`, as `group_threads` in
    `rail.rs` does for thread rows).
  - `script/e2e/491-marley-mcp.sh` (the stand-in client; `own_terminal()` guesses its terminal
    from the running command), `script/e2e/501-zeds-agents-drive-the-browser.sh` (the stand-in
    ACP agent, set as a custom agent server in the run's settings).
  - Claude Code 2.1.283's bundle: the stdio server's `env` is `{...Pe, CLAUDE_PROJECT_DIR, …,
    ...r.env}` where `Pe` is the process environment unless `CLAUDE_CODE_MCP_ALLOWLIST_ENV` is
    set (then an allowlist).
- **Decisions:** D1 to D8 in the spec.

### Design
- **Approach.**
  1. `marley_terminal::identity` (new): `TERMINAL_ID_VARIABLE`, `PROJECT_VARIABLE`,
     `RESTORED_ID_KEY` (private to the handoff), `new_terminal_id()` (`uuid::Uuid::new_v4`),
     `is_terminal_id(&str)`.
  2. `TerminalBuilder::new`, beside the nonce: take `RESTORED_ID_KEY` out of `env`; for a task or
     a remote terminal remove both variables; for a local interactive one set
     `TERMINAL_ID_VARIABLE` to the restored id when it is well formed, else a new one. The
     `Terminal` keeps the id (`marley_terminal_id: Option<String>` and its getter), set in the PTY
     literal and `None` in the display-only one.
  3. `create_terminal_shell_internal`: after `env.extend(settings.env)`, a local terminal gets
     `PROJECT_VARIABLE` from `first_project_directory` (removed when there is none, and when the
     project itself is remote, as for `create_local_terminal` in a remote project, whose folder is
     not on this machine), and a restored id goes in under `RESTORED_ID_KEY`. The private function takes the id as a new
     parameter; `create_terminal_shell` passes `None`; the new `create_terminal_shell_restoring`
     passes the saved one.
  4. The hook: `terminal_view::MarleyTerminalIdentity`, a global of three functions (the saved id
     for an item, save an item's id, clean up the items not loaded). `deserialize` reads the id
     beside the working directory and calls `create_terminal_shell_restoring`; `serialize` saves
     the terminal's id with the rest; `cleanup` also calls the hook's cleanup. With no hook set
     (Zed's own tests), all three do nothing.
  5. `marley_workbench::terminal_ids` (new): `MarleyTerminalIdsDb`, table
     `marley_terminal_ids(workspace_id, item_id, terminal_id)`, keyed by the pair, with the
     workspace foreign key and cascade #494's table has; sets the hook at `init`.
  6. The bridge: `caller_headers()` reads the two variables and `os.getcwd()`, keeps each well
     formed value, and `request` adds them to every POST.
  7. `marley_mcp`: `HttpRequest` gains the three values (`header_value` on `marley-terminal`,
     `marley-project`, `marley-cwd`, each capped); a `Caller { terminal, project, cwd }` (all
     `Option<String>`) rides from `serve_connection` to `ask_app` into `AppCall::new` and out
     through `AppCall::caller()`.
  8. `mcp.rs`: `terminal_list` adds `terminal_id` and `self`; `terminal_argument` falls back to
     the caller's terminal (by `marley_terminal_id`); the context server's `env` gains both
     variables set to empty strings.
  9. `browser.rs` and `browser_tools.rs`: the hub keeps its focus history (targets, most recent
     last) instead of the last one alone, and `focused_in(project, cx)` picks the newest whose
     tab's workspace is in the project's group; `page_of` and `navigate` take the caller's group
     from `caller_project(caller, cx)` (terminal, then `Marley-Project`, then `Marley-Cwd`);
     `place_tab` opens a new tab in that group's last active workspace; `tabs` adds `project` and
     `default` per tab.
  10. `registry.rs`: the descriptions of `terminal_list`, `terminal_blocks`, `terminal_read` and
      the browser tools say what a call with no `terminal` or `tab` acts on.
- **File manifest.**
  - Marley: `crates/marley_terminal/src/identity.rs` (new),
    `crates/marley_terminal/src/marley_terminal.rs`, `crates/marley_terminal/Cargo.toml`
    (`uuid`), `crates/marley_workbench/src/terminal_ids.rs` (new),
    `crates/marley_workbench/src/marley_workbench.rs` (module and `init`),
    `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge`,
    `crates/marley_mcp/src/transport.rs`, `crates/marley_mcp/src/marley_mcp.rs`,
    `crates/marley_mcp/src/registry.rs`, `crates/marley_workbench/src/mcp.rs`,
    `crates/marley_workbench/src/browser_tools.rs`, `crates/marley_workbench/src/browser.rs`,
    `script/e2e/520-terminal-identity.sh` (Test).
  - Zed: `crates/terminal/src/terminal.rs` (the builder hunk, the field and its getter),
    `crates/project/src/terminals.rs` (the project variable, the restored id,
    `create_terminal_shell_restoring`), `crates/project/Cargo.toml` (`marley_terminal`),
    `crates/terminal_view/src/terminal_view.rs` (the hook global and its three calls).
  - Generated: `Cargo.lock`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: new rows for
  `crates/project/src/terminals.rs` and `crates/project/Cargo.toml`; the rows for
  `crates/terminal/src/terminal.rs` and `crates/terminal_view/src/terminal_view.rs` gain #520's
  hunks, with the merge advice to keep the identity beside the nonce and the hook calls in
  serialize, deserialize and cleanup.

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | setup exports `MARLEY_TERMINAL_ID=inherited` and `MARLEY_PROJECT=/nowhere`; `ids` in A's terminal | `520-01-identity` |
| REQ-002 | split A's terminal (Zed's split, clicked); `ids` in the new one | `520-01-identity` |
| REQ-005 | `mcp list` in A's first terminal | `520-02-self` |
| REQ-006 | `mcp blocks` with no terminal named | `520-03-own-blocks` |
| REQ-003 | open B through Zed's Open (path typed); `task: spawn` the `.zed/tasks.json` task | `520-04-task` |
| REQ-007 | the rail's + in B, New Browser Tab, a page there; click A's terminal row; `mcp navigate <page two>` | `520-05-scoped-tab` |
| REQ-008 | #501's stand-in ACP agent from B's + menu, prompted "open <page three>" | `520-06-agent-panel`; the stand-in's log |
| REQ-009 | the harness-side client with an empty environment: `terminal_list`, then `browser_navigate` | the run log: no `self`; the page loads in the tab the user focused last, whichever its project |
| REQ-004 | `quit_marley`, `launch_marley`; `ids` in A's restored terminal | `520-07-restored`; the run log compares the id lines |

What no scenario reaches: Claude Code itself handing the bridge its environment (the bundle says
it does; Test adds one live check, `claude --plugin-dir` in a pty calling `terminal_list` once,
if the promotion wants the proof beyond the stand-in client).

### Risks
- The widest Zed touch of this batch: four files. Each hunk sits beside an existing Marley one
  (the nonce, the footer hook) or is a new function; the rows carry the merge advice.
- Zed's Open with its own path prompt may put B in a new window instead of the rail's window; the
  scenario then opens B from the rail's add-project button.
- The hub's focus history must forget closed pages, or `focused_in` could name a gone tab
  (`page_state` guards the single value today, and the list gets the same filter).
- A shell that sets its own `MARLEY_TERMINAL_ID` can pose as another terminal. The id is a
  convenience for scoping, not authority: the MCP bearer still gates every call.
- If the slice runs long, the restore (REQ-004: the hook and the table) splits off, and this
  slice keeps a per-launch id.
