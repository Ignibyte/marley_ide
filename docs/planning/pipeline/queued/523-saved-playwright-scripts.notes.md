# Saved Playwright scripts, run on a Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-523-saved-playwright-scripts.md
- **Pipeline spec:** 523-saved-playwright-scripts.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "playwright that can run scripts inside marley itself. that
  someone can keep saved inside marley." The prompt for this spec: a library of Playwright scripts
  kept in Marley (per project, and global), run against Marley's own Chromium through its
  DevTools endpoint, acting on the chosen Browser tab while the user watches, output in a
  terminal block, the flight recorder catching the last minute on a failure; find what Node and
  Playwright the box has, and how a script gets the endpoint and the tab's target id.
- **Classification / tier:** feature, prong 3. Marley crates, one embedded Node program, one e2e
  scenario; no Zed path. Size M.
- **Recall (§18.3):**
  - AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001: no tool evaluates
    script an agent supplies. Runs are the user's act (D4), and no MCP tool runs a script.
  - AD-claude-488 and plan D16: any CDP client finds the endpoint in the profile's
    `DevToolsActivePort`; Playwright was named as one of them.
  - L-claude-488-an-overlay-from-any-session-shows-in-every-screencast-001: what one CDP
    session draws shows in every session's frames, so the user sees Playwright's effects in the
    tab through Marley's own screencast.
  - L-claude-499-a-screencast-sends-frames-only-when-the-page-changes-001: a failed run's
    recording holds frames only where the page changed; its timeline, not a frame count, says how
    long the run took.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: the Scripts button moves
    the toolbar's buttons; #496's scenario clicks the pick button by coordinates and is rerun.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001: not needed by the tray,
    which is buttons, not a menu.
  - Brain: a page on the Playwright MCP ("an agent-drivable browser means a Chromium you
    control"); nothing on saved scripts.
- **Discovery:**
  - Node and Playwright on the box (read only, 2026-09-25): `node`, `npx` and `npm` at
    `~/.local/share/mise/installs/node/latest/bin/`, which resolves to 26.9.0 (installs 26.8.1
    and 26.9.0); `pnpm` and `deno` in `/usr/bin`; `~/.local/bin/playwright` is a bash wrapper
    (`mise use -g --quiet "npm:playwright"`, then `mise x "npm:playwright" -- playwright`), with no
    `npm:playwright` among mise's installs, so its first run would install Playwright and change
    mise's global config. `playwright-core` 1.63.0 sits only in npx's cache
    (`~/.npm/_npx/e41f203b7505f1fb/node_modules/`, from `npx playwright`), with 1.63.0-alpha and
    1.61.0-alpha beside the Playwright MCP 0.0.80 and 0.0.75; `~/.cache/ms-playwright/` holds
    Chrome for Testing builds 1234 and 1243. `playwright-core` 1.63.0: `engines.node >= 20`, 14 MB,
    no `scripts` in its `package.json`, and `browsers.json` pins Chromium 153.0.8010.12. The
    Arch package is Chromium 152.0.7977.82.
  - `crates/marley_browser/src/service.rs`: `endpoint_in` (212) reads the port and the browser's
    WebSocket path from `DevToolsActivePort`, which gives `MARLEY_CDP_URL` as
    `ws://127.0.0.1:<port><path>`.
  - `crates/marley_workbench/src/browser.rs`: `render_toolbar` (4064) with the pick (4166),
    annotate (4185) and record (4205) buttons; `render_tray` (4438), the picks tray under the
    toolbar; `BrowserHub::record` (1387), which takes the page's minute and saves it off the main
    thread; `record_this` (3761), its toast; `recordings_dir` (2641); `agent_ended` (1626), which
    shows how an entry reaches the page's minute (`record_entry`); `place_tab` (2798), whose busy
    branch splits a pane to the right and keeps the focus (the shape the run's terminal takes).
  - `crates/marley_workbench/src/agents.rs`: `start_cli` (188): `TerminalPanel::add_center_terminal`
    (196) with the launcher's factory, the startup handshake with `STARTUP_TIMEOUT` (45, five
    seconds), then `write_init_command_after_startup` (211); `marley_agent::send_payload` (84)
    turns a line into the bytes that run it.
  - `crates/terminal/src/terminal.rs`: `blocks` (1845); the integration is loaded only when
    `task.is_none()` (1224); `wait_for_completed_task` (3284) for task terminals.
    `crates/marley_terminal/src/block.rs`: `BlockState` (29: Pending, Running, Finished),
    `ExitCode` (41).
  - `crates/marley_browser/src/recorder.rs`: `Entry` (41), with `Agent { did }` (102) as the
    closest kind; `save_in` (347).
  - `crates/marley_workbench/src/mcp.rs`: `write_bridge_in` (171) writes the embedded bridge
    (`claude_plugin::BRIDGE`, `claude_plugin.rs:28`) into the data folder, mode 0755.
  - `crates/paths/src/paths.rs`: `config_dir` (123; `<custom data dir>/config` under the e2e's
    `--user-data-dir`), `tasks_file` (309), `snippets_dir` (379).
- **Decisions:** D1 to D7 in the spec.

### Design
- **The runner** (`crates/marley_workbench/playwright/run.mjs`, embedded with `include_str!` and
  written to `<data>/playwright/run.mjs` when its bytes differ; Marley's own file, `MIT OR
  Apache-2.0`; Playwright is a runtime dependency, nothing of it copied). It imports
  `chromium` from `playwright-core` (resolved from `<data>/playwright/node_modules`), reads
  `MARLEY_CDP_URL` and `MARLEY_TAB` (a clear message and exit 2 when either is missing),
  `connectOverCDP`, takes `browser.contexts()[0]`, and finds the page whose
  `Target.getTargetInfo` (through `context.newCDPSession(page)`) names `MARLEY_TAB`, waiting up to
  five seconds for it. It imports the script by its file URL, checks that the default export is a
  function, prints `▶ <name> on <url>`, awaits it with `{ page, context, browser }`, then prints
  `✓ <name> passed in <s> s` and exits 0, or the error's message and stack and exits 1. It never
  closes the page, the context or the browser; it disconnects in a `finally`.
- **The template** a new script starts from: the function, a comment naming what it receives and
  the two variables, one `getByRole` line as an example, and a note not to close the page, the
  context or the browser and not to set the viewport, which is the tab's.
- **`playwright.rs`.** `library(project_key) -> Vec<Script { name, scope, path }>` reads both
  folders (`.mjs` files, sorted by name); `create(scope, key, name)` turns the name into a file
  name (letters, digits, `-` and `_` kept, anything else becomes `-`), refuses one that exists,
  writes the template and returns the path; `ensure_runner(data)` writes `run.mjs` and
  `package.json`; `command(endpoint, target, script, installed) -> String` quotes every path with
  `shlex` (single quotes) and prefixes `npm install --prefix <data>/playwright --no-audit
  --no-fund && ` when `node_modules/playwright-core` is absent. `run(view, script, window, cx)`:
  resolves the tab's endpoint (`service::endpoint_in` of the tab's project profile), opens a
  center terminal of the tab's workspace in a pane split right of the tab's pane (the tab kept in
  front), waits for the startup handshake as `start_cli` does, writes the command, and watches
  the terminal: on each notify it reads `blocks()`, takes the first block, and when it is
  `Finished` reads its exit code. A non-zero code calls `BrowserHub::record` for the tab's page
  and shows the toast; `ExitCode(None)` shows a toast saying Marley could not tell how the run
  ended (no shell integration). The `Script` entries go into the page's minute when the command
  is written and when the block finishes.
- **The tray.** A Scripts button (`IconName::PlayOutlined`, tooltip "Playwright scripts") after
  the record button; it toggles the tray, which draws one row per script (name, a scope chip,
  Run with `IconName::PlayFilled`, Edit with `IconName::FileCode`, which opens the file in an
  editor tab of the workspace) and a last row with a name field (a single-line `Editor`, as the
  pick captions are) and two buttons, "For this project" and "For all projects"; Enter in the field
  is "For this project". A name that exists shows an error under the field. `marley::PlaywrightScripts`
  toggles the tray on the focused Browser tab.
- **The recorder.** `Entry::Script { name: String, exit_code: Option<i32> }`, serialized like the
  others (`kind: "script"`), `exit_code` absent at the start. `browser_recording` returns it as it
  returns every entry.
- **File manifest.** Marley crates: `crates/marley_workbench/src/playwright.rs` (new),
  `crates/marley_workbench/playwright/run.mjs` and `template.mjs` (new, embedded),
  `crates/marley_workbench/src/browser.rs`, `marley_workbench.rs`;
  `crates/marley_browser/src/recorder.rs`. Script: `script/e2e/523-saved-playwright-scripts.sh`.
  No Zed path.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`. `.config/typos.toml` may need the
  runner's folder if typos flags anything in it; check at P2.

### E2E plan
Setup writes the site, the scratch HOME (`.bashrc`: the directory of the setup's own
`command -v node` first on the PATH, then the scenario's prompt), the three scripts into
`$E2E_PROFILE/config/playwright/projects/<key>/` and `…/global/` (the key from the repository's
path, #507's derivation), and copies `playwright-core` 1.63.0 from the first
`~/.npm/_npx/*/node_modules/playwright-core` whose `package.json` says 1.63.0 into
`$E2E_PROFILE/playwright/node_modules/`. Steps open the browser on `login.html` through the
palette and the address bar, then click.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | click the Scripts button | `523-01-tray`: `log-in` and `broken` marked for the project, `where` for every project |
| REQ-002 | click the name field; type `check title`; click "For all projects" | `523-02-new`: `check-title.mjs` open in an editor tab with the template; the log lists `global/` |
| REQ-003 | click the Browser tab's tab; open the tray; click `log-in`'s Run; settle eight seconds | `523-03-passed`: the terminal beside the tab, its block's `✓ log-in passed` and exit status |
| REQ-004 | the same view | `523-03-passed`: the page reads "Signed in as Marley", drawn at the tab's size |
| REQ-005 | Run `where`; `mcp_agent tabs` | the log: `where.mjs` prints the tab's id, equal to `browser_tabs`' id, and the page's URL |
| REQ-007 | `mcp_agent recordings` before and after `log-in` | the log: no new recording |
| REQ-006 | Run `broken`; settle eight seconds; `mcp_agent recordings`, then `recording <id>` | `523-04-failed`: the block's failure and the toast; the log: the timeline's two `script` entries, the second with exit code 1 |
| REQ-008 | not in the scenario: it would fetch from the npm registry | Test: Chad's first run on his machine installs into `~/.local/share/marley/playwright/`; the notes record the block |

### Risks
- `connectOverCDP` may apply emulation to the pages it attaches (a viewport, a device scale),
  which would fight the tab's own `Emulation.setDeviceMetricsOverride`. P1 checks with a scratch
  Chromium before Code; the template says not to set the viewport; REQ-004's shot shows the tab
  still at its size.
- Playwright 1.63.0 targets Chromium 153 and Marley's is 152; the attach is documented as lower
  fidelity than Playwright's own launch. P1's check covers the calls the template uses; a later
  Arch Chromium closes the gap.
- Playwright dismisses a JavaScript dialog that the script does not handle, so a page's dialog
  during a run closes before the user can answer it on the tab's card. The template's comment
  says so.
- A script may open pages of its own; each gets a Browser tab by #493's rules, behind the focus.
  One that closes the default context would close every page of the project's Chromium; the
  template warns, and nothing stops a script from doing it.
- Two first runs at once would run two installs into one folder. The run refuses to start a
  second install while one runs and says so in a toast.
- A shell whose PATH has no Node 20 or newer fails the block (exit 127 or a version error) and
  saves a recording that shows nothing useful; the toast names the block's message instead of
  the recording when the exit code is 127.
- The command line carries the CDP endpoint in the block's header. It is no secret today (CDP
  takes no credential, plan D15); #524's second slice, which puts a token in front of CDP, must
  give the runner its token without printing it.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
