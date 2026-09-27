# Programs that open a browser land in Marley's Browser tab, through BROWSER — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-561-browser-env-opener.md
- **Pipeline spec:** 561-browser-env-opener.spec.md

## Phase 1 — Plan
- **Request:** the Orca second pass of 2026-09-25, finding 3 ("Programs that open a browser land
  in Marley's Browser tab"), ranked third: a small opener beside the bridge, exported as
  `BROWSER` in the environment Marley's local shells get, sending the URL and its working
  directory to Marley's MCP endpoint with the bearer; Marley applies #503's rule. Chad decided
  on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Classification / tier:** feature, S once #503 has landed. Marley crates (`marley_workbench`,
  `marley_mcp`, `marley_terminal`, the plugin's `bin/`); one Zed touch, the existing hunk in
  `crates/terminal/src/terminal.rs` widened by one variable, its row updated.
- **Recall (§18.3):**
  - AD-claude-491 (the MCP server in the app): wire names are `family_verb`; the app's families
    answer deferred, on the main thread; `browser_open_url` follows `browser_navigate`'s route.
  - AD-claude-492: write tools are granted at start and the client's approval of each call is a
    check; the opener is its own client and approves nothing, which is why the tool opens
    Browser tabs only (D4).
  - PR-claude-474 and #519 D4: a frame in terminal output is display data; an action that turns
    text into input checks authenticity first. The opener carries the bearer for that reason.
  - L-claude-493: a tab added with the focus takes it, which is what an auto-open wants; the
    reuse case activates the old tab.
  - #513 (one Marley per data directory): the endpoint file is written durably and removed by
    its owner; the e2e fixture points `MARLEY_MCP_ENDPOINT` at the profile copy's.
  - The ledgers hold nothing on `BROWSER` or `xdg-open` (grepped 2026-09-26). Brain: not consulted
    in this drafting pass; promotion runs `brain_ask`.
- **Discovery (opened and checked, 2026-09-26):**
  - `crates/marley_terminal/src/shell_integration.rs`: `ZSH_ZDOTDIR_VARIABLE` (38),
    `MARKER_VARIABLE` (41), `NONCE_VARIABLE` (44), `new_nonce` (48-50), the directory (81-83),
    `for_program` (89-121: bash gets the marker and `--rcfile`, zsh `ZDOTDIR` and the marker;
    other shells nothing).
  - `crates/terminal/src/terminal.rs`: `marley_shell_integration` (89-122, `env.extend` at 113);
    `insert_zed_terminal_env` (716-725); `TerminalBuilder::new` (1153-1168); `SHLVL` removed
    (1196); `LANG` added only if absent (1202-1205, the one `or_insert`); the nonce hunk
    (1209-1219, `!is_remote_terminal` at 1212); the integration hunk (1221-1228: unix, no task, not
    remote, a PTY); `clone_builder` (3385-3403, a split reuses the environment). The test at
    4564-4610 checks the nonce is absent for a remote terminal.
  - `vendor/alacritty_terminal/src/tty/unix.rs:271-283`: the PTY layer adds `USER`, `HOME`,
    `WINDOWID`, applies the map, removes `XDG_ACTIVATION_TOKEN`; the child inherits the rest of
    Marley's environment.
  - `crates/project/src/terminals.rs`: `create_terminal_shell` (284-290),
    `create_terminal_shell_internal` (312-451, `is_via_remote` at 319, `TerminalBuilder::new` at
    409-424), `resolve_directory_environment` (583-605), `settings.env` (378),
    `create_remote_shell` (608-642, `insert_zed_terminal_env` at 615 for the remote shell).
  - `crates/util/src/shell_env.rs:117-128` (`$SHELL -l -i -c` capture) and
    `crates/util/src/util.rs:401-423` (`load_login_shell_environment`, `crates/zed/src/main.rs:455-459`).
  - `/usr/share/omarchy/default/bash/envs:8`: `export BROWSER="${BROWSER:-omarchy-launch-browser}"`,
    with the comment that a session-wide `BROWSER` stops `xdg-settings` changing the default
    browser; reached from `~/.bash_profile:5` through `~/.bashrc:15`.
  - `crates/marley_workbench/src/claude_plugin.rs`: `FILES` (35-65), `MANIFEST` (32),
    `shipped_version` (108-112), the directory `paths::data_dir().join("claude-code")` (125),
    `write_plugin_in` (170-182), `write_marketplace` (262-281), install (283-306), update (343-370).
  - `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge`: `POLL_SECONDS` (28),
    `REQUEST_TIMEOUT` (31), the loopback hosts (33), the endpoint file (53-60), its checks (63-83),
    SSE and JSON replies (94-110), the request (113-142), the handshake (194-218), the retry on
    404 (220-233), DELETE (266-274).
  - `crates/marley_workbench/src/mcp.rs`: the caller channel (71-77), the grant of `browser.write`
    (84-92), the endpoint written (94-100) and removed on quit (114-120), the foreground task
    (125-130), the context-server entry with `MARLEY_MCP_ENDPOINT` (178-184), the bridge's second
    copy (208-219), `write_endpoint` (222-237), `answer` (317-329), the workspace walk (333-361).
  - `crates/marley_mcp/src/transport.rs`: `spawn` (113-132), the bearer (93-100, 120), the checks
    (177-181), `ask_app` (280-292), the SSE stream (271-276), the request parser (395-429), the
    endpoint JSON (487-491); `session.rs:152-173` (32 sessions, 30 minutes idle);
    `discovery.rs:16, 28-41`; `dispatch.rs:18-42, 133-174` (`Deferred` at 165-172);
    `marley_mcp.rs:81` (`APP_CALL_TIMEOUT_SECONDS`), 156-163 (`AppCall::answer`).
  - `crates/marley_mcp/src/registry.rs`: the browser rows (221-241), `browser_navigate` (191-195),
    its schemas (731-759), `tab` (348-354).
  - `crates/marley_workbench/src/browser_tools.rs`: `answer` (53-61), `browser_navigate` (430-471,
    `agent_url` at 442, `new_page` at 448). `crates/marley_browser/src/address.rs`: `agent_url`
    (29-38), `url_for` (42-60), `host_kind` (65-104).
  - `crates/marley_workbench/src/browser.rs`: `PageOpened` (748-768), `open_tab` (2754-2793),
    `place_tab` (2798-2874), `active_multi_workspace` (2730-2738), `reveal` (2903-2926),
    `new_page` (2174-2182), `new_view` (2740-2749), `open_page_in` (2960-2984), `new_tab`
    (5744-5754), `reveal_terminal` (5430-5459), `show_for_agent` (5681-5688); the shared profile at
    627 (#507 changes it).
  - `crates/project/src/project.rs:4938-4944` (`find_worktree`);
    `crates/workspace/src/multi_workspace.rs:1060-1077` (`workspace_for_paths`, an exact set,
    not containment), `workspaces` (1287), `activate` (1310).
  - `crates/gpui_linux/src/linux/platform.rs:447-449, 816-867`; the `open` crate 5.3.2
    (`unix.rs:8-32`; `lib.rs:82`: `BROWSER` "is ignored ... unless the opener being used happens
    to respect it").
  - On the box: `gh` 2.101.0; Python 3.14.7's `webbrowser.py` (566-589, 50-57, 173-199);
    `/usr/bin/xdg-open` (1024-1048, 1073-1122, 1191-1205); `opener` 0.8.5 in the registry
    (`src/lib.rs:87-115`); `xdg-mime query default x-scheme-handler/http` → `google-chrome.desktop`.
  - Orca (MIT, read at `1c2cf120e3`): `assembly.ts:33-38, 291-298`; `browser-basic.ts:18-23`;
    `browser-tab.ts:18-27`; `selectors.ts:175-201`; `browser-tab-create-params.ts:20-23` (the
    worktree required); `runtime-browser-commands-browser-tab-create.ts:42-46, 189-213`;
    `browser-tab-create-publication.ts:134-150` (selected on the client, no host focus).
- **Decisions:** D1 to D7 in the spec.

### Design
- **Approach.**
  - *The opener* (`bin/marley-open-url`, about 120 lines): `main(url)`; `endpoint()` as the
    bridge's (the file, the loopback check, `url` and `headers.Authorization`); `call(url,
    directory)`: three POSTs with `urllib.request` and a 5 s deadline for the whole exchange
    (`initialize` with the opener as the client name, `notifications/initialized`, `tools/call`
    `browser_open_url`), the `Mcp-Session-Id` carried, a DELETE at the end in a `finally`; the
    tool's result parsed from JSON or an SSE `data:` line, as the bridge reads replies. `opened`
    true exits 0. Anything else: `fallback(url)`: `os.environ.pop("BROWSER")`, `os.execvp("xdg-open",
    ["xdg-open", url])`; `xdg-open` missing prints one line to stderr and exits 1. The bearer is
    never printed.
  - *Shipping:* a `FILES` row (`bin/marley-open-url`, program); `mcp::start` writes it beside
    the bridge's second copy (`<data_dir>/mcp/marley-open-url`, 0755) from `include_str!`, and
    removes nothing on quit (a shell may still hold the path; a missing endpoint is the fallback).
  - *The variable:* `marley_terminal::shell_integration::browser_opener_in(data_dir) -> PathBuf`
    and a `ShellIntegration` entry `("BROWSER", <path>)` for every program, not only bash and
    zsh; `for_program` gains a `browser: Option<PathBuf>` argument the terminal hunk fills from a
    new `Global` the workbench sets at start and on a settings change (`MarleyTerminalEnv {
    browser_opener: Option<PathBuf> }` in `terminal`, beside the hunk; `None` under
    `system_browser`). The hunk's one new line reads the global. Both live inside the existing
    `// Marley:` block (1221-1228), so the row in the ledger widens and no new row is needed.
  - *The tool:* `registry.rs`: `browser_open_url` (write, `browser.write`), the input schema
    `{url: string, directory: string}` (`additionalProperties: false`), the output `{opened:
    boolean, tab?: string, project?: string, reason?: string}`; `dispatch` defers it as the other
    browser tools. `browser_tools::browser_open_url`: `address::agent_url` (http and https, else
    `opened: false, reason: "not http or https"`); `links::route` for a non-SSH local program
    under `MarleySettings::terminal_links`; the project by `directory`: every window's
    workspaces (`mcp.rs:333-361`'s walk), `project.find_worktree(directory)` on each, the deepest
    root wins, none is `reason: "no project holds the directory"`; then `browser::open_url_tab`
    (#503) in that workspace, and the workspace activated in its window as `reveal_terminal`
    does; the answer names the tab's target id and the project.
  - *The setting:* `system_browser` clears the global, so terminals started after the change
    carry no `BROWSER`; ones started before keep the path, and the tool declines everything, so
    the opener falls back: consistent either way.
- **File manifest.** Marley crates: `crates/marley_workbench/claude_plugin/marley/bin/marley-open-url`
  (new), `crates/marley_workbench/src/claude_plugin.rs` (`FILES`), `mcp.rs` (the copy, the global),
  `browser_tools.rs`, `browser.rs` (if #503's `open_url_tab` needs a window-activation variant),
  `links.rs` (#503's), `crates/marley_mcp/src/registry.rs`, `crates/marley_terminal/src/shell_integration.rs`.
  Zed crate: `crates/terminal/src/terminal.rs` (the existing hunk). Scripts:
  `script/e2e/browser-fixture.sh` (`mcp_agent open-url <url> <directory>`),
  `script/e2e/561-browser-env-opener.sh`.
- **Ledger row (`docs/marley/zed-touchpoints.md`, before the edit).** The
  `crates/terminal/src/terminal.rs` row gains: the #463 hunk also inserts `BROWSER` from
  `MarleyTerminalEnv` for local terminals (#561). Why: programs that open a browser read it, and
  Marley's opener routes them to a Browser tab. On merge: keep the line inside the hunk; if
  upstream grows a terminal-environment hook, move the variable there and shrink the row.
- **Knowledge at Complete (expected).** An AD for D2 (export always, files override, the reason
  from Omarchy) and D4 (the tool opens tabs only).

### E2E plan
Setup: `offline_chromium`; `serve_site site` for a page titled "Opened page"; `$E2E_WORK/bin`
with the fake `xdg-open` and `user-browser` (bash, `printf '%s\n' "$1" >> <log>`); the HOME's
`.bashrc` sets `PS1='$ '` and `PATH="$E2E_WORK/bin:$PATH"`; `$E2E_WORK/elsewhere`; the scratch
repository opened. `mcp_agent open-url <url> <directory>` calls the tool through the bridge, as
the other `mcp_agent` verbs do.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `echo "$BROWSER"` | `561-01-env`: `<profile data dir>/mcp/marley-open-url` |
| REQ-002 | `python3 -c 'import webbrowser; webbrowser.open("http://127.0.0.1:<port>/")'`; settle 4 | `561-02-tab`: the Browser tab on "Opened page", focused; the log's `mcp_agent tabs` |
| REQ-003 | back to the terminal's tab; the same call | `561-03-same-tab`: one Browser tab, in front |
| REQ-004 | `webbrowser.open("https://example.com/docs")`, then `webbrowser.open("file://$E2E_WORK/doc.html")` | `561-04-system`: no new tab; `xdg-open.log` holds both |
| REQ-005 | `cd $E2E_WORK/elsewhere; python3 -c '…the local URL…'` | `561-05-outside`: no new tab; the log line |
| REQ-006 | `cd -; BROWSER=$E2E_WORK/bin/user-browser python3 -c '…'` | `561-06-user-browser`: `user-browser.log` holds the URL, `xdg-open.log` unchanged |
| REQ-008 | in the scenario's own shell: `time MARLEY_MCP_ENDPOINT=$E2E_WORK/missing.json $E2E_PROFILE/mcp/marley-open-url http://127.0.0.1:<port>/` with the fake first on the PATH | the run log: one more `xdg-open.log` line, well under a second |
| REQ-009 | `mcp_agent open-url https://example.com/ $E2E_WORK/repo` | the run log: `opened: false`, the reason; `xdg-open.log` unchanged |
| REQ-007 | `quit_marley`; the settings written; `launch_marley`; `echo "$BROWSER"`; the local URL again | `561-07-off`: not the opener; the log's new line |

Not reachable by a scenario: a real `gh` or Vite (Python's `webbrowser` is the reference reader,
and `gh` reads the same variable the same way); the real system browser (faked by `xdg-open`);
a remote project's terminal (read in the review through `is_remote_terminal`).

### Risks
- Omarchy's `BROWSER` is set per shell from bash's files, with `${BROWSER:-…}`, so Marley's value
  survives; a `.bashrc` that exports it unconditionally overrides Marley's, which is D2's intent.
  The scenario's HOME shows the plain case and the override.
- `xdg-open` on this desktop goes to the scheme handler (`google-chrome.desktop`), never to
  `BROWSER`, so the fallback opens what the desktop's default is: the same as before this ticket.
- Five seconds while Marley is busy (a modal, a long main-thread task): the user sees the system
  browser instead of a tab, and the opener logs nothing; the tool's own answer is what the run
  log proves. A longer wait would hang the program that called the opener.
- tmux inside a Marley terminal keeps the environment its server started with; a shell in an old
  server carries an old `BROWSER`, which the opener's fallback handles.
- Vite reads `BROWSER` from `.env` too; a project's `.env` wins over the environment there.
- #520's `Marley-Cwd` may later let the tool default `directory`; the opener keeps passing it, so
  nothing here changes when #520 lands.
- The bridge's second copy and the opener sit under `<data_dir>/mcp/`; the profile copy an e2e
  run makes has its own, so the scenario's `$BROWSER` names the copy's path (`561-01-env`).

## Promotion (2026-09-26, at `e83b5e943c`)
- **What has landed since the draft:**
  - #503: `links.rs` (`destination`, `over_ssh`), `browser::open_url_tab` and
    `address::local_url`.
  - #520: the caller headers the bridge sends (`Marley-Terminal`, `Marley-Project`,
    `Marley-Cwd`), with `MARLEY_TERMINAL_ID` and `MARLEY_PROJECT` in every local terminal.
  - #574: `browser_tools::holding(path)`, the local workspace one of whose folders holds a path,
    the longest winning, which is D5's rule as written.
  - Omarchy still exports `BROWSER="${BROWSER:-omarchy-launch-browser}"` from its bash envs.
- **Seams re-read:**
  - `terminal.rs`: `TerminalBuilder::new` (1155), the nonce hunk (1212-1221), #520's and #575's id
    hunk (1223-1248), the integration hunk (1250-1257, no task).
  - `project/src/terminals.rs`: `MARLEY_PROJECT` and `MARLEY_RESTORED_TERMINAL_ID` inserted after
    `settings.env` (393-410).
  - `mcp.rs`: `start` (66-110, `browser.write` granted), `offer_to_zeds_agents` (179-235, the
    bridge written off the main thread), `write_bridge_in` (238-249).
  - `claude_plugin.rs`: `BRIDGE` (29), `FILES` (35-65).
  - The bridge: `endpoint_path` (53-60), `read_endpoint` (63-83), `messages_in` (94-110),
    `caller_headers` (117-137).
  - `registry.rs`: the browser write rows (188-224), `browser_write` (238-246),
    `browser_write_schemas` (743).
  - `browser_tools.rs`: `answer` (71-81), `caller_scope` (86-111), `holding` (114-131).
  - No scenario pins the tool count while Marley runs (491's only count is "lists 0 tools" with
    no Marley).
- **Corrections to the design:**
  1. **The opener's endpoint is the one beside it.** Terminals carry no `MARLEY_MCP_ENDPOINT`: only
     the context server's bridge gets it. The bridge's default,
     `~/.local/share/marley/mcp-endpoint.json`, is the user's main Marley, so a scratch Marley's
     terminal, or an e2e run, would open its tabs in the user's own Marley. The opener lives at
     `<data_dir>/mcp/marley-open-url` and reads `<data_dir>/mcp-endpoint.json`, the file its own
     Marley writes; `$MARLEY_MCP_ENDPOINT` still wins when set.
  2. **The path goes through a process-wide setting in `marley_terminal`, not a gpui global.**
     `marley_terminal` has no gpui, so `shell_integration::set_browser_opener` and
     `browser_opener` keep an `RwLock<Option<PathBuf>>`. The workbench sets it at start and on a
     settings change: none under `system_browser`. The builder's one new hunk inserts `BROWSER`
     for every terminal that is not remote, tasks included (a dev server started as a task opens
     its page too), so it sits beside the id hunk rather than inside the integration hunk, which
     skips tasks. The ledger row for `terminal.rs` widens.
  3. **The opener's source is outside the plugin.** It is not one of Claude Code's plugin files
     (nothing in the plugin runs it), so it lives at `crates/marley_workbench/bin/marley-open-url`,
     is written by `mcp.rs` beside the bridge's second copy, and `FILES` does not change.
  4. **The answer is `{opened, project?, reason?}`.** A new tab's page id is not known until its
     page is made, and the opener needs only `opened`.
- **Safety in the scenario (L-claude-503-gpui-falls-back…).**
  - Python's `webbrowser` tries `BROWSER` first and then every browser it knows, `xdg-open` first
    and the real browsers after, until one exits 0. Under `system_browser` the terminal's
    `BROWSER` would be the captured login environment's `omarchy-launch-browser`, which opens the
    user's real browser.
  - So the scenario sets `terminal.env BROWSER` to its fake `xdg-open`. That value is overridden
    by the opener while it is exported and stands when it is not.
  - Fakes named `xdg-open`, `gio` and `google-chrome` go first on both PATHs. Each logs and exits
    0, the last two to `leak.log`, which must stay empty.
- **Recall added:** AD-claude-503 (the route and `open_url_tab`);
  L-claude-503-gpui-falls-back-to-the-desktop-portal… (the fake exits 0);
  PR-claude-empty-a-variable-the-child-must-not-inherit-001 (a removed key is inherited, so
  under `system_browser` the inherited value stands, which is the desktop default this ticket
  wants).
- **Brain consultation e345fed38cdf47c18328a1ac3e78d5cc:** nothing on this seam (only unrelated follow-ups due). A second, identical ask (bcbc368b…) was closed with `brain no-decision` as a repeat.

## Phase 2 — Code
- **Built:**
  - `marley_terminal::shell_integration`: `BROWSER_VARIABLE`, and `set_browser_opener` and
    `browser_opener` over a process-wide `RwLock<Option<PathBuf>>`.
  - `crates/terminal/src/terminal.rs`, two hunks. Before the future, the opener is read and
    dropped when `TerminalSettings`' `env` names a `BROWSER` of the user's own. After #520's id
    hunk, it is inserted as `BROWSER` for every terminal that is not remote, tasks included.
  - `crates/marley_workbench/bin/marley-open-url` (new, Python 3, standard library):
    - the endpoint is `$MARLEY_MCP_ENDPOINT`, else `mcp-endpoint.json` beside its own data
      directory, loopback only;
    - `initialize`, `notifications/initialized`, then `tools/call browser_open_url {url,
      directory}` and a DELETE, all within 5 s;
    - `opened: true` exits 0, and anything else execs `xdg-open` with `BROWSER` removed;
    - the failures caught: `OSError`, `ValueError`, `http.client.HTTPException` and `URLError`.
  - `mcp.rs`:
    - `offer_browser_opener` sets the path at once and again on each settings change, none
      under `system_browser`, and writes the file off the main thread;
    - `write_program_in` now writes both the bridge and the opener.
  - `registry.rs`: `browser_open_url` (write, `browser.write`) and `open_url_schemas`.
  - `browser_tools.rs`: `open_url`, answered before the browser is up, since a new tab waits for
    it by itself. It runs `agent_url`, then `links::browser_tab_url`, then `holding(directory)`,
    then `window_of` (now `pub(crate)`). Through the `AnyWindowHandle` it shows the workspace in
    its window and then opens the tab (`open_url_tab`), in separate updates, and answers
    `{opened, project}` or `{opened: false, reason}`.
  - `links::browser_tab_url`: #503's `destination` with no SSH and no key held.
  - `browser-fixture.sh`: `mcp_agent open-url <url> <directory>`.
- **Deviations from the plan, and why:**
  1. The promotion's corrections: the endpoint beside the opener, the setting in
     `marley_terminal`, the source outside the plugin, and the answer without `tab`.
  2. A `BROWSER` in the user's `terminal.env` wins over the opener. It is the user's explicit
     setting, and `terminal.env` is merged in three places in `project/src/terminals.rs`, while
     the builder already has `TerminalSettings`.
  3. `open_url` answers before `showing()`. Waiting for a stopped browser to start could pass
     the opener's 5 s, and the opener would then send the URL to the system browser while the
     tab opened too.
  4. The workspace is shown and the tab added through `AnyWindowHandle::update`, whose root is
     not leased. A `WindowHandle<MultiWorkspace>` update would have the tab added inside the
     root's update.
- **Review:**
  - The bearer is never printed or logged, and goes to loopback only.
  - The fallback's environment is the process's without `BROWSER`, through `os.execvpe`, with no
    shell anywhere.
  - Nothing opens the system browser from Marley's side.
  - The registry's schemas are closed (`additionalProperties: false`).
  - Routing: `mcp::answer` hands every `browser_` call to `browser_tools::answer`.
  - No scenario pins the tool count.
  - Provenance: Orca's relay (MIT) is reimplemented, and the endpoint and SSE code follows
    Marley's own bridge.
  - Checks: `cargo check -p marley_workbench` is clean, fmt is clean, and `cargo clippy -p
    marley_terminal -p terminal -p marley_mcp -p marley_workbench --all-targets -- -D warnings`
    is clean.

## Phase 3 — Test
- **The scenario:** `script/e2e/561-browser-env-opener.sh` (`compositor sway`, offline Chromium,
  a served page "Opened page").
  - Python's `webbrowser` is the program that reads `BROWSER`.
  - Fakes named `xdg-open`, `user-browser`, `gio`, `google-chrome` and `firefox` go first on the
    runner's PATH, which Marley inherits, and on the terminal's. Each logs and exits 0, the last
    three to `leak.log`.
  - The scenario's `.bashrc` sets `BROWSER` to the fake `xdg-open` unless it is the opener.
  - 16 `expect` checks, through `terminal-read`, `mcp_agent tabs` and `open-url`, the three logs
    and a timed run of the opener.
- **Green, the debug build, first run:** all 16 pass. The fallback with no endpoint took 30 ms,
  and `leak.log` stayed empty.
- **Red, the installed build (`e83b5e943c`, before the change):** `check BROWSER is Marley's
  opener: FAIL`, since the terminal carried the `.bashrc`'s fake.
- **Shots, read:**
  - `561-01-env`: `echo "$BROWSER"` prints `/run/user/1000/marley-e2e/profile.…/mcp/marley-open-url`,
    the profile copy's opener (REQ-001).
  - `561-02-tab`: an "Opened page" Browser tab of `repo` at `http://127.0.0.1:<port>/`, in front;
    `tabs` says `project repo, focused` (REQ-002).
  - `561-03-same-tab`: after the same call, one Opened page tab, in front (REQ-003).
  - `561-04-system`: after the docs URL and the `file://` page, the terminal in front and no new
    tab; `xdg-open.log` holds both (REQ-004).
  - `561-05-outside`: from `elsewhere`, a folder in no project, the local URL reached
    `xdg-open.log` and opened no tab (REQ-005).
  - `561-06-user-browser`: with `BROWSER=…/user-browser` on the command line,
    `user-browser.log` holds the URL (REQ-006).
  - `561-07-off`: after `marley.terminal_links` became `system_browser`, a new terminal's
    `BROWSER` is the fake and not the opener, and its local URL reached `xdg-open.log` with no
    tab (REQ-007).
  - The run log: the opener with a missing endpoint handed the URL to `xdg-open` in 30 ms
    (REQ-008). `browser_open_url` with `https://example.com/` answered `{"opened": false,
    "reason": "marley.terminal_links sends this URL to the system browser"}`, and nothing
    reached the log (REQ-009).
  - The footers in `561-04` to `561-07` offer `127.0.0.1:<port>`. That is #503's rule: the typed
    command lines hold the site's URL, and its port listens.
- **Not reached by a scenario:** real `gh` and Vite, which read the same variable the same way,
  and a remote project's terminal (the builder's `!is_remote_terminal`).
- **The golden set, with 561 added (`just regress`, the debug build):** all 23 pass. #503 still
  passes on the same route, and #492 still passes with the registry's new tool. 561 took 83 s.
- **The focus report:** each sway run stopped with its Marley, pointer and keyboard. The
  Hyprland check found no Marley window before or after the run, and nothing was added or
  reloaded.
- **The gate:** `script/gates.sh --diff` gave `GATE GREEN [diff]`, 16 passed and 0 failed,
  shellcheck and semgrep on the new scenario and the fixture among them. The receipt matches
  the tree.
- **Verdict:** PASS. REQ-001 to REQ-007 are shown by the shots and the checks, and REQ-008 and
  REQ-009 by the run log's checks.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md`, under Added: "Programs that open a browser open a Browser tab".
  - `docs/marley/three-prong-plan.md`: prong 3's table gains B6c (#561, shipped).
  - `docs/marley_architecture/marley_mcp.md`: `browser_open_url`, and eight write tools.
  - `docs/marley_architecture/marley_workbench.md`: the opener, under Terminal links.
  - The `crates/terminal/src/terminal.rs` row was checked against what shipped: the opener read
    before the future and skipped for a `terminal.env` `BROWSER`, and the insert for every
    terminal that is not remote.
- **Knowledge appended:**
  - `F-claude-561-an-opener-on-the-default-endpoint-would-open-tabs-in-another-marley-001`
  - `PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001`
  - `L-claude-561-pythons-webbrowser-tries-every-browser-it-knows-after-a-failed-one-001`
  - `AD-claude-561-marley-exports-its-opener-as-browser-in-every-local-terminal-001`
- **Brain:** consultation `e345fed38cdf47c18328a1ac3e78d5cc` was closed by `brain decide`
  (`decisions/marley-exports-its-opener-as-browser-in-every-local-terminal-and-the-opener-finds-the-marley-that-wrote-it`,
  follow-up 2026-10-26); the repeat ask was closed with `brain no-decision`.
- **Open for Chad:** the spec's question on `file://` pages from the project (`cargo doc
  --open`, coverage reports). The default shipped: they go to the system browser.
- **Closed:** TICKET-561 moved to `tickets/closed/`, and its BACKLOG row went at promotion.
