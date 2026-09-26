---
pipeline_id: 59e32e7e-bf9f-46ff-89e5-4a0f8dcff2c6
ticket: docs/planning/tickets/open/TICKET-561-browser-env-opener.md
status: Phase 4 — Complete PASS
title: "Programs that open a browser land in Marley's Browser tab, through BROWSER"
type: feature
slice: prong 3 with prong 1; a follow-up slice of #503 (the Orca second pass, finding 3); after #503
references: [docs/planning/design-notes/orca-second-pass-2026-09-25.md, docs/planning/pipeline/queued/503-terminal-urls-open-in-the-browser.spec.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/marley/three-prong-plan.md]
---

## Title
A program that opens a URL itself (`gh pr view --web`, Vite's `--open`, Python's `webbrowser`, a
Jupyter server) reads `BROWSER`, the Unix convention for the program that opens URLs. Marley
ships a small opener, exports it as `BROWSER` in every local terminal it starts, and the opener
asks Marley over the MCP endpoint with the bearer: a local URL opens in a Browser tab of the
project whose folder holds the program's working directory, with the focus, by #503's rule;
anything else the opener hands to `xdg-open` itself, with `BROWSER` unset, so nothing loops
back. Under #507 that tab is in the project's own Chromium, where an agent can then drive the
page.

## Scope
### In
- **The opener**, `marley-open-url` (Python 3, standard library only, beside the bridge:
  `crates/marley_workbench/claude_plugin/marley/bin/marley-open-url`, shipped as a program in
  `claude_plugin::FILES` and written, like the bridge's second copy, to `<data_dir>/mcp/marley-open-url`
  at start with mode 0755). It takes one argument, the URL; reads the endpoint file as the bridge
  does (`$MARLEY_MCP_ENDPOINT`, else `~/.local/share/marley/mcp-endpoint.json`; http on loopback
  only); opens an MCP session (`initialize`, `notifications/initialized`), calls
  `browser_open_url {url, directory}` with its working directory, and ends the session. An
  answer of `opened` exits 0. A decline, no endpoint file, a refused call or five seconds
  without an answer runs `xdg-open <url>` with `BROWSER` removed from the environment and exits
  with its status.
- **The tool** `browser_open_url {url, directory}` (the browser family, write tier, grant
  `browser.write`, granted at start): applies #503's route for a program in `directory`, under
  `marley.terminal_links` (`local_in_browser_tab`: a local http or https URL by #503 D2;
  `all_in_browser_tab`: any http or https URL; `system_browser`: nothing), never for SSH (the
  opener runs locally). The project is the one whose folder is the deepest holding `directory`
  among every window's workspaces (`Project::find_worktree`); a Browser tab of that workspace
  already on the URL comes forward, else a new tab opens after the active item with the focus
  (#503's `open_url_tab`), and the workspace is shown in its window. `0.0.0.0` and `[::]` open as
  loopback (#503 D2). The answer is `{opened: true, tab, project}` or `{opened: false, reason}`
  (not local, no project holds the directory, `system_browser`, not http or https). It never
  opens the system browser.
- **The variable**: every local terminal Marley starts gets `BROWSER=<data_dir>/mcp/marley-open-url`,
  in the environment `TerminalBuilder::new` builds (the existing Marley hunk,
  `crates/terminal/src/terminal.rs:1221-1228`, through `marley_terminal::shell_integration`), for
  every shell, unless `marley.terminal_links` is `system_browser`, when nothing is exported. A
  remote terminal's `ssh` gets none, as it gets no nonce and no integration.
- `script/e2e/561-browser-env-opener.sh`.

### Out (explicitly deferred)
- Programs that open through `xdg-open` (Node's `open` package, Electron apps): catching them
  takes a Marley `.desktop` handler for `x-scheme-handler/http`, which would replace the
  desktop's default browser, more than this is worth.
- `file://` pages from the project (`cargo doc --open`, a coverage report): the note's open
  question 2, Chad's; the default keeps them in the system browser (plan D15 limits agent
  navigation to http and https). Listed under the open question below.
- Sign-ins (`gh auth login --web`): they reach the real browser through the rule, since the login
  page is never local, and an OAuth redirect to `localhost` happens inside that browser.
- A caller identity on the tool (#520's headers): when #520 has landed, `directory` may default
  to the caller's `Marley-Cwd`; the opener still passes its own.
- A shell started before the endpoint file exists (a terminal restored before `mcp::start` wrote
  it): the opener falls back that once; no retry.
- Remote (SSH) projects and shells, whose remote shell never sees the variable; Windows and macOS.

## Reference (§20)
Orca's `BROWSER` relay (the second pass, finding 3; report 03 §2.3 on link routing):
`src/main/ipc/pty/host-env/assembly.ts:291-298` sets `BROWSER="orca open-url --url %s"` in the
headless runtime's terminals when neither the terminal's base environment nor Orca's process has
one; `orca open-url` (`src/cli/specs/browser-basic.ts:18-23`, `src/cli/handlers/browser-tab.ts:18-27`)
resolves the worktree from its working directory, and `browserOpenUrlOnClient`
(`src/main/runtime/runtime-browser-commands-browser-tab-create.ts:189-213`) takes http and https
only and selects the new tab on the paired client. Marley keeps the mechanism (the variable, the
working directory choosing the project, http and https only) and changes two things: the opener
is a plain path taking the URL as its argument, since Rust's `opener` runs the whole value as a
program and Python substitutes `%s` only when present; and Marley exports it in every local
terminal, since Omarchy sets `BROWSER` for every shell on this box, so "only when unset" would
never fire, while a shell's own files still override it. Upstream Zed opens every URL with the
platform's `open_url` (`gpui_linux`'s `open_uri_internal`, the `open` crate's `xdg-open`) and
sets no `BROWSER`. Warp's docs (files-and-links, cited in #503): web links open in the default
browser; nothing on `BROWSER`.

### Prior art
- **Behavior maps and reports.** The second pass, finding 3: the readers (`gh`: `GH_BROWSER`,
  then `BROWSER`; Python's `webbrowser`; Vite's `--open`, also from `.env`; `cargo doc --open`);
  `xdg-open` takes the desktop's `x-scheme-handler/http` default first and reads `BROWSER` only
  when that fails; the bearer instead of an in-band frame, which any program's output could
  print; the loopback rule for sign-ins; `file://` as open question 2. Report 03 §2.3 and report
  05 §2.4 (link routing, SSH panes on the system browser), which #503 took. Orca's files, read at
  `1c2cf120e3`, with the lines above; `assembly.ts` is never called for SSH terminals (33-38), and
  the flag is set only for the headless runtime (`orca-runtime-sync-window-graph.ts:25-27`).
- **Published material.** `gh help environment` (gh 2.101.0 on the box: "GH_BROWSER, BROWSER
  (in order of precedence): the web browser to use for opening links"). Python 3.14's
  `webbrowser` (`/usr/lib/python3.14/webbrowser.py:566-589`: `BROWSER` split on `:`, each entry
  tried first; `GenericBrowser` (173-199) gives a plain command the URL as its only argument and
  substitutes `%s` when present). `/usr/bin/xdg-open` (`open_envvar` 1024-1048: `%s` substituted,
  else `$browser "$1"`; `open_generic` 1103-1109: the scheme handler's default first, `BROWSER`
  second; this desktop counts as `generic`, and `xdg-mime query default x-scheme-handler/http`
  answers `google-chrome.desktop`, so `BROWSER` is never reached through `xdg-open` here).
  Rust's `opener` 0.8.5 (`open_browser`, `src/lib.rs:87-115`: `Command::new(&browser_var).arg(path)`,
  no `:` split, no `%s`). MCP 2025-06-18's Streamable HTTP (a session from `initialize`,
  `Mcp-Session-Id`, DELETE to end it), which the bridge already speaks.
- **The code we already ship.** `crates/terminal/src/terminal.rs`: `TerminalBuilder::new`
  (1153-1168), `insert_zed_terminal_env` (716-725, `TERM_PROGRAM=zed`, overwriting), the nonce
  hunk (1209-1219, skipped for a remote terminal at 1212), the integration hunk (1221-1228)
  calling `marley_shell_integration` (89-122, `env.extend` at 113); `crates/marley_terminal/src/shell_integration.rs`:
  `for_program` (89-121), the variable names (38-44). `crates/project/src/terminals.rs`:
  `create_terminal_shell_internal` (312-451), the environment from `resolve_directory_environment`
  (583-605, a `bash -l -i` capture, which is how Omarchy's `BROWSER` gets in) and `settings.env`
  (378); `create_remote_shell` (608-642). The plugin: `claude_plugin::FILES` (35-65),
  `write_plugin_in` (170-182, 0755 for programs); the bridge's second copy at
  `<data_dir>/mcp/marley-mcp-bridge` (`mcp.rs:208-219`); the bridge itself
  (`bin/marley-mcp-bridge`: the endpoint file 53-60, its loopback check 63-83, the request shape
  113-142, the handshake 194-218, `REQUEST_TIMEOUT` 40). The server: `transport.rs` `spawn`
  (113-132, `127.0.0.1:0` and the bearer), the bearer check (177-181, constant time), sessions
  (`session.rs:152-173`), `ask_app` (280-292) with `APP_CALL_TIMEOUT_SECONDS` 30; there is no
  plain route, every POST is JSON-RPC in a session, so the opener speaks MCP. `registry.rs`: the
  browser family's write rows with `browser.write` (233-241), `browser_navigate`'s schema
  (753-759: `url`, `new_tab`, `tab`; no project). `browser_tools.rs:430-471` (`browser_navigate`;
  `address::agent_url` at 442, http and https only, plan D15; `new_page` at 448). `browser.rs`:
  `new_tab` (5744-5754), `open_page_in` (2960-2984), `new_page` (2174-2182), `new_view`
  (2740-2749), `place_tab` (2798-2874), `reveal_terminal` (5430-5459, a workspace shown in its
  window); #503's `open_url_tab` and `links::route` (queued). `Project::find_worktree`
  (`crates/project/src/project.rs:4938-4944`, a prefix test); `mcp.rs:333-361` walks every
  window's workspaces; `marley_browser::address::host_kind` (65-104) and #503's `local_url`.
  `crates/gpui_linux/src/linux/platform.rs`: `open_url` (447), `open_uri_internal` (816-867, the
  `open` crate 5.3.2: `xdg-open`, `gio open`, `gnome-open`, `kde-open`). On the box:
  `/usr/share/omarchy/default/bash/envs:8`, `export BROWSER="${BROWSER:-omarchy-launch-browser}"`,
  so Marley's process environment and every captured directory environment carry it (read
  2026-09-26). Does a crate we build own the seam? The bridge owns the client side (the opener
  copies its thirty lines for the endpoint), `marley_mcp` the server, #503 the route and the
  tab; nothing owns the variable, one line in an existing hunk.

## UI proof
UI-AFFECTING (a Browser tab opens; the setting's effect). `script/e2e/561-browser-env-opener.sh`
(Hyprland, keys only; `offline_chromium`; a site served by `serve_site`). Setup: a scratch
repository opened; a HOME whose `.bashrc` puts `$E2E_WORK/bin` first on the PATH, holding a fake
`xdg-open` that appends its argument to `xdg-open.log` and a `user-browser` that appends to
`user-browser.log`; `$E2E_WORK/elsewhere`, a folder in no project; `doc.html`. Shots, each
after a command typed in the project's terminal: `561-01-env` (`echo "$BROWSER"` prints the
opener's path under the profile copy's data directory); `561-02-tab` (`python3 -c 'import
webbrowser; webbrowser.open("http://127.0.0.1:<port>/")'`: a Browser tab of the project on the
page, with the focus); `561-03-same-tab` (the same call: one Browser tab, in front);
`561-04-system` (`https://example.com/docs`, then `file://$E2E_WORK/doc.html`: no new tab, both
in `xdg-open.log`); `561-05-outside` (`cd $E2E_WORK/elsewhere` and the local URL: no new tab,
the log line); `561-06-user-browser` (`BROWSER=$E2E_WORK/bin/user-browser python3 -c …`:
`user-browser.log` holds the URL); `561-07-off` (`quit_marley`, `"marley": {"terminal_links":
"system_browser"}` in the profile copy's settings, `launch_marley`; `echo "$BROWSER"` is not the
opener, and the local URL lands in `xdg-open.log`). The run log: `mcp_agent tabs` after
`561-02-tab` (one tab on the URL), the two logs, the opener run by hand with
`MARLEY_MCP_ENDPOINT` naming a missing file and timed (under a second, one `xdg-open.log` line),
and the stand-in agent's own `browser_open_url` call with `https://example.com/` (declined, no
log line), all checked with `expect`.

## Locked-In Decisions
- D1: The opener is a plain executable path that takes the URL as its only argument, never a
  `%s` template: `opener` runs the value as a program name, Python appends the URL when there is
  no `%s`, and `gh` and `xdg-open` take both forms.
- D2: Marley exports `BROWSER` in every local terminal it starts, as it sets `TERM_PROGRAM`, and a
  shell's own files can override it. Orca sets it only when nothing else has; on this box
  Omarchy exports `BROWSER=omarchy-launch-browser` from every login shell and Zed's environment
  capture carries it into each terminal, so that rule would never fire here. What the desktop's
  value did, launching the default browser, the opener still does for every URL it declines.
  `system_browser` turns the export off.
- D3: The opener speaks MCP with the bearer from the endpoint file, as the bridge does: the
  server has no other route, and the bearer is what keeps a printed frame from steering the
  browser (#519 D4, PR-claude-474). A tab opened this way is the tool's doing, so it needs
  `browser.write`, granted at start.
- D4: The tool applies #503's rule and opens Browser tabs only; the system browser stays the
  opener's own fallback, run with `BROWSER` unset. An agent calling `browser_open_url` gets what
  `browser_navigate` with `new_tab` gives it, placed in a project; it gains no way to open Chad's
  real browser.
- D5: The working directory chooses the project, the deepest folder holding it among every
  window's workspaces (#521's rule for ports); no project, the opener falls back. Rejected: "the
  tab the user focused last", which #520 exists to fix.
- D6: `file://` URLs are declined by default (plan D15; the note's open question 2, Chad's);
  `0.0.0.0` and `[::]` open as loopback (#503 D2).
- D7: Five seconds for the whole exchange, then the fallback: a program waiting on its opener
  (Python's `GenericBrowser` waits for it to exit) must not hang on a stuck Marley.

**Open question for Chad** (the note's question 2): should `file://` pages from the project
(`cargo doc --open`, coverage reports) open in a Browser tab too? Default: no, they go to the
system browser; plan D15 keeps agent navigation to http and https.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.terminal_links` is not `system_browser`, a local terminal Marley starts shall have `BROWSER` set to the opener's path. | Shot `561-01-env` |
| REQ-002 | WHEN a program in a project's terminal opens a local http URL through `BROWSER`, the system shall open a Browser tab of that project on the URL, with the focus, within five seconds. | Shot `561-02-tab`; the run log's `mcp_agent tabs` |
| REQ-003 | WHEN the project already has a Browser tab on that URL, the system shall bring it forward instead of opening another. | Shot `561-03-same-tab` |
| REQ-004 | WHEN the URL is not local, or is not http or https, the opener shall hand it to `xdg-open` with `BROWSER` unset and open no tab. | Shot `561-04-system`; `xdg-open.log` |
| REQ-005 | WHEN the program's working directory lies in no open project, the opener shall hand the URL to `xdg-open`. | Shot `561-05-outside`; the log |
| REQ-006 | WHERE the user's shell exports its own `BROWSER`, programs shall use it. | Shot `561-06-user-browser`; `user-browser.log` |
| REQ-007 | WHERE `marley.terminal_links` is `system_browser`, terminals shall not carry the opener and every URL shall reach the system browser. | Shot `561-07-off`; the log |
| REQ-008 | WHEN Marley is not running or does not answer within five seconds, the opener shall hand the URL to `xdg-open` and exit. | The run log: the opener run by hand against a missing endpoint file, timed |
| REQ-009 | WHEN `browser_open_url` is called with a URL the rule declines, the system shall answer `opened: false` with the reason and open nothing. | The run log: the stand-in agent's call with `https://example.com/`; `xdg-open.log` unchanged |

## Phase Plan
- **P1 Plan:** this spec; #503 ships first. At promotion, check that Omarchy still exports
  `BROWSER` (`/usr/share/omarchy/default/bash/envs`) and whether #520's headers landed.
- **P2 Code:** the row for `crates/terminal/src/terminal.rs` in `docs/marley/zed-touchpoints.md`
  first (the #463 hunk widened by the variable); the opener and its `FILES` entry, the second
  copy; `browser_open_url` in the registry and `browser_tools`, the workspace-by-directory
  lookup; the export in `marley_terminal::shell_integration`; fmt and clippy clean; a review of
  the diff (the bearer never logged, the fallback's environment, no shell anywhere).
- **P3 Test:** write and run the scenario and read every shot; rerun #503's scenario (the same
  route) and `492-browser-tools.sh` (golden; the registry gains a tool); `script/gates.sh --diff`
  green.
- **P4 Complete:** CHANGELOG; the plan's prong 3; `docs/marley_architecture/marley_workbench.md`
  and `marley_mcp.md`; the touchpoints row checked against what shipped; the ledger capture;
  close the ticket, archive, commit.
