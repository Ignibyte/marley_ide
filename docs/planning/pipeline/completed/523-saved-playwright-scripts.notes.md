# Saved Playwright scripts, run on a Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-523-saved-playwright-scripts.md
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

### Promotion (2026-09-27, at f99a866e8e)
- **What landed since the draft:** #507 (a Chromium and a profile per project: the key is
  `service::project_key`, a tab's is `BrowserView::project`, and its profile
  `service::profile_in(&service::project_dir_in(paths::data_dir(), key))`, `browser.rs:1151`);
  #581 (Clear Browser Data); #505 and #506 (the check card and the recorder's `Action` entries).
  D1's condition is met: the key exists.
- **Seams re-verified** (an Explore sweep at f99a866e8e):
  - `browser.rs`: `render_toolbar` (5077): back, forward, reload or stop, the address bar
    (`flex_1`), then pick (5141), annotate (5142) and record (5143) against the right edge; the
    Agent chip after them while an agent acts. `render_tray` (5573), the picks tray under the
    toolbar at 5906, its rows (`render_pick_row`, 5648) with a caption `Editor` per row
    (`caption`, 4547) and buttons wired through `cx.listener`. `BrowserHub::record(target, dir,
    project, cx) -> Task<Result<String>>` (2090), `recordings_dir()` (3440), `record_this`
    (4766) with its toast. `record_entry` (2036) pushes into the page's minute while a tab draws
    it. `place_tab`'s busy branch (3694) splits a pane right and puts the focus back.
    `init` (6600) registers the browser actions; the keymap is
    `crates/marley_workbench/keymap.json` (`MarleyBrowser`, 43).
  - `service.rs`: `endpoint_in(profile)` (385) gives the port and the browser's path.
  - `recorder.rs`: `Entry` (237), `#[serde(tag = "kind", rename_all = "snake_case")]`,
    Serialize only; `Recorder::push` (497) merges fills, typing and scrolls, and stores the rest
    as they come; `take` (617) times them from the earliest; `save_in` (727); `read_in` (808).
  - `agents.rs`: `start_cli` (188): `TerminalPanel::add_center_terminal` (always the active pane;
    `terminal_panel.rs:848`), the startup handshake with `STARTUP_TIMEOUT` (five seconds), then
    `write_init_command_after_startup`. No Marley code opens a terminal in a split beside another
    item.
  - Blocks: `Terminal::blocks()` (`terminal.rs:1915`, `AnchoredBlock`s); a hook only calls
    `cx.notify()`, so a watcher observes the terminal. A block opens Running and finishes with
    its `ExitCode(Option<i32>)` (`anchored.rs:209`). No production code waits for a block yet;
    the test helper `finished_block` (`terminal.rs:4819`) is the model, with `cx.observe` for its
    polling.
  - `mcp.rs:280` `write_program_in` writes an embedded program into the data folder, mode 0755,
    at every launch (no comparison).
  - `paths.rs`: under `--user-data-dir`, `data_dir()` is that folder and `config_dir()` its
    `config/`.
  - The toolbar by coordinates: 490 (`FORWARD_X=303`, `RELOAD_X=329`), 496 (`PICK_X=1288`), 505
    (`PICK_X`, `RELOAD_X=334`), 518 (`PICK_X`). A button between the address bar and Pick moves
    none of them: the buttons after it keep the right edge.
  - Quoting: `util::shell::ShellKind::try_quote` (`util/src/shell.rs:436`, `shlex` for POSIX
    shells), through the `util` dependency Marley has.
- **Checked on the box (the plan's own check):** a stand-in for Marley held a flat session on a
  scratch offline Chromium's page with a 900×700 `Emulation.setDeviceMetricsOverride`;
  `playwright-core` 1.63.0 (rustal's copy, linked read-only) attached with `connectOverCDP`,
  found the page by `Target.getTargetInfo`'s id, saw 900×700 with `page.viewportSize()` null,
  filled through `getByLabel` and clicked through `getByRole`, and `browser.close()` only
  disconnected: the page stayed open, the same target, still 900×700. The draft's first risk is
  closed.
- **The first run's install is reachable:** `npm install --prefix <dir> --no-audit --no-fund`
  with `npm_config_registry` on a loopback server that serves a `playwright-core` packument and
  the tarball `npm pack` made of rustal's copy installed 1.63.0 in 195 ms, three requests and
  no network. REQ-008 moves into the scenario.
- **Recall at promotion:** AD-claude-492 (no agent runs script in a page: D4), AD-claude-499
  (the minute is kept while a tab draws the page: the run keeps the tab in front), AD-claude-506
  (a recording's actions and the drafter: a `script` entry is one more kind the drafter skips),
  AD-claude-507 (the key and the per-project Chromium: the run attaches to the tab's project's
  endpoint), L-claude-498 (a toolbar button moves older clicks: the button goes left of Pick),
  L-claude-506 (prove a generator by running what it makes: the scenario runs the template's
  script, and a new script from the tray). Brain consultation b96a02155cb74feca776547f7c48a76d:
  nothing on this seam.

### Design (revised at promotion)
- **The runner** (`crates/marley_workbench/playwright/run.mjs`, embedded with `include_str!`,
  written to `<data>/playwright/run.mjs` with `package.json` before each run, as
  `write_program_in` writes the bridge). It imports `chromium` from `playwright-core` (resolved
  from `<data>/playwright/node_modules`), reads `MARLEY_CDP_URL` and `MARLEY_TAB` (a message and
  exit 2 when either is missing), `connectOverCDP`, takes `browser.contexts()[0]`, and finds the
  page whose `Target.getTargetInfo` names `MARLEY_TAB`, waiting up to five seconds. It imports
  the script by its file URL, checks the default export is a function, prints `▶ <name> on
  <url>`, awaits it with `{ page, context, browser }`, then prints `✓ <name> passed in <s> s` and
  exits 0, or the error and exits 1. In a `finally` it calls `browser.close()`, which on a
  connected browser only disconnects (the check above).
- **The template** (`crates/marley_workbench/playwright/template.mjs`): the default export, a
  comment naming what it receives and the two variables, a commented `getByRole` line, and a
  note to leave the page, the context, the browser and the viewport as they are.
- **`playwright_scripts.rs`** (new; the name keeps it apart from `marley_browser::playwright`,
  #506's drafter). Pure: `file_name(name)` (letters, digits, `-` and `_`; anything else a `-`;
  empty refused), `command(endpoint, tab, runner, script, install)`, quoted with
  `ShellKind::Posix.try_quote`. `*_in(dir)` IO: `library_in(config, key) -> Vec<Script { name,
  scope, path }>` (the project's then the global `.mjs` files, sorted by name), `create_in(config,
  scope, key, name) -> PathBuf` (refuses a name that exists; writes the template),
  `write_runner_in(data)`, `installed_in(data)` (`node_modules/playwright-core/package.json`
  there). All of it off the main thread.
- **The run.** `run_script(view, script, window, cx)`: reads the tab's endpoint
  (`service::endpoint_in` of its project's profile), writes the runner, checks the install; with
  none it refuses when another first run installs (a toast). It splits the tab's pane right
  (`Workspace::split_pane`, the new pane active) and opens a center terminal there
  (`TerminalPanel::add_center_terminal`), waits for the startup handshake as `start_cli` does,
  writes the command, puts `Script { name, exit_code: None }` in the page's minute, and observes
  the terminal until its first block finishes. A code of 0 ends it; a non-zero code puts the
  `Script` entry with the code in the minute, calls `BrowserHub::record` for the tab's page and
  shows a toast naming the recording; `ExitCode(None)` shows a toast that Marley cannot tell how
  the run ended (a shell without its integration).
- **The tray.** A Scripts button (`IconName::PlayOutlined`, tooltip "Playwright Scripts") between
  the address bar and Pick. It toggles the tray under the toolbar, above the picks tray: a row per
  script (its name, a chip for its scope, Run, disabled while the tab has no page, and Edit, which
  opens the file in an editor tab of the workspace), then a name field (a single-line `Editor`)
  with "For This Project" and "For All Projects"; Enter in the field is "For This Project". An
  error shows under the field. `marley::PlaywrightScripts` toggles the tray on the focused Browser
  tab. The list is read when the tray opens and after a script is made.
- **The recorder.** `Entry::Script { name: String, exit_code: Option<i32> }` (`kind: "script"`,
  `exit_code` left out at the start). The drafter (#506) passes over it.
- **File manifest.** Marley crates: `crates/marley_workbench/src/playwright_scripts.rs` (new),
  `crates/marley_workbench/playwright/run.mjs` and `template.mjs` (new, embedded),
  `crates/marley_workbench/src/browser.rs`, `marley_workbench.rs`, `keymap.json` (none needed
  unless a key is wanted: none), `crates/marley_browser/src/recorder.rs`. Scripts:
  `script/e2e/523-saved-playwright-scripts.sh`, `script/e2e/golden`. At Complete:
  `CHANGELOG.md`, the plan's slice row, `marley_browser.md` (the entry), `marley_workbench.md`
  (the tray and the run).
- **Ledger rows.** None: no Zed path changes.

### E2E plan
`compositor sway`, the offline Chromium. Setup: a loopback site whose `login.html` has a Name
field and a Sign in button that shows "Signed in as <name>"; a scratch HOME whose `.bashrc` puts
the box's Node first on the PATH and points npm at a loopback registry (`npm_config_registry`,
`npm_config_cache` in the scratch HOME, `npm_config_update_notifier=false`) that serves
`playwright-core` 1.63.0 packed from rustal's copy (read-only; setup stops, naming it, when no
copy is found); the project's `log-in.mjs` and `broken.mjs` and the global `where.mjs` written
into `$E2E_PROFILE/config/playwright/`. Marley opens the repository; `marley: open browser`,
`login.html`; the tray's buttons are clicked at coordinates measured from the first run.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | click the Scripts button | `523-01-tray`: `log-in` and `broken` for the project, `where` for all projects |
| REQ-002 | click the name field, type `check title`, click "For All Projects" | `523-02-new`: `check-title.mjs` open with the template; the log lists `global/` |
| REQ-008, REQ-005 | back in the tab, the tray, Run on `where` (the first run: no install yet) | the log: the block (`terminal-read`) shows npm's install, then `where`'s `MARLEY_TAB` equal to `browser_tabs`' id and the page's URL |
| REQ-003, REQ-004, REQ-007 | Run on `log-in` | `523-03-passed`: the terminal beside the tab, the block's `✓ log-in passed`, the page "Signed in as Marley" at the tab's size; the log: no new recording |
| REQ-006 | Run on `broken` | `523-04-failed`: the block's failure and the toast naming the recording; the log: the recording's timeline with two `script` entries, the second with exit code 1 |

Not reachable by a scenario: nothing. A shell without Marley's integration (`ExitCode(None)`)
is Out's fish case, which no scenario starts.

### Risks
- Playwright 1.63.0 targets Chromium 153 and the box's is 152; the calls the template and the
  scenario use worked in the check above.
- Playwright dismisses a JavaScript dialog a script does not handle; the template says so.
- A script may open pages of its own; each gets a Browser tab by #493's rules. One that closes the
  default context closes every page of the project's Chromium; the template warns.
- Two first runs at once would install twice into one folder: the second is refused while the
  first installs.
- A shell with no Node 20 or newer fails the block (127, or a version error); the toast shows the
  block's end rather than the recording's name when the code is 127.
- The command carries the CDP endpoint, which is no secret today (plan D15); #524's token must
  reach the runner without printing it.

## Phase 2 — Code
- **Checklist** (no task tool): the runner and the template ✓; `playwright_scripts.rs` ✓; the
  recorder's entry ✓; the tab's button, tray and run ✓; the actions and the keymap entry ✓; fmt ✓;
  check and clippy with `--all-targets` ✓ (no cargo ran while the release install's golden set
  ran); the review ✓.
- **Built.** `crates/marley_workbench/playwright/run.mjs` (the runner: `connectOverCDP`, the page
  by `Target.getTargetInfo`'s id, the script's default export, `▶`, `✓` or `✗` with the stack,
  exit 1 on a throw, `browser.close()` in a `finally`, which disconnects) and `template.mjs`.
  `playwright_scripts.rs`: `Scope`, `Script`, `library_dir_in`, `library_in`, `file_stem`,
  `create_in` (`create_new`, so an existing name is refused), `write_runner_in`, `installed_in`,
  `command` (`ShellKind::Posix.try_quote` for every part), `RunTarget`, `run`: the endpoint,
  the runner and the install check off the main thread; an `Installing` global refuses a second
  first run; `open_terminal`; the startup handshake and the command; the `Script` entries;
  `block_end` (the first block, polled every 250 ms, ten seconds for it to open); the recording
  and the toasts. `recorder.rs`: `Entry::Script { name, exit_code }`. `browser.rs`:
  `BrowserHub::record_script`; `ScriptsTray` (`Option`, open while `Some`), `toggle_scripts`,
  `read_scripts`, `new_project_script`, `create_script`, `edit_script`, `run_script` (which
  focuses the tab first, so its pane is the active one when the terminal is made); the Scripts
  button between the address bar and Pick; `render_scripts_tray` with `render_script_row` and
  `render_new_script`, drawn above the picks tray. `marley_workbench.rs`: the module and the
  actions `PlaywrightScripts` and `NewPlaywrightScript`; `keymap.json`: Enter in
  `MarleyScriptName > Editor`.
- **Deviations from the plan, and why.**
  - The run's terminal: `add_center_terminal` always adds to the active pane, and Zed's
    `set_active_pane` is private; copying `add_center_terminal`'s body into a Marley crate is out
    (§20). So the tab takes the focus first, the terminal is made in its pane, and then it moves:
    into the pane right of the tab's when there is one (`find_pane_in_direction`,
    `move_active_item`), so later runs reuse it, else into a new split (`split_and_move`).
  - The block is polled rather than observed: the run is an async task, and a poll every 250 ms
    reads the same `blocks()`.
  - A new action, `NewPlaywrightScript`, gives the name field Enter, as `SendPick` gives a
    caption's.
  - Opening a script's file reports a failure in a prompt (`detach_and_prompt_err`), not the log.
- **Review of the diff.** REQ-001: the tray lists the project's then the global scripts, each
  row saying which. REQ-002: `create_in` writes the template under `create_new` and the file opens.
  REQ-003 and REQ-004: the terminal ends beside the tab, the tab in front; the command is one
  line. REQ-005: the command's environment is the endpoint and the target id. REQ-006 and
  REQ-007: a non-zero code saves the minute and names it; zero saves nothing. REQ-008: the
  install is prefixed while `node_modules/playwright-core` is missing, and a second first run is
  refused. Blocking IO runs on the background executor only (`future::lazy`, for the dylint
  lints). No entity is read while it is updated. No Zed path changed; AD-claude-492 holds: no
  agent tool runs a script.
- **Found in the release install, outside #523:** the golden set on the release build failed
  #507's and #581's scenarios on a title the login site sets after its IndexedDB read. Chromium
  sends no event for a script's `document.title` (checked on a scratch Chromium 152), so Marley
  saw the title only when its load-time read came late enough, as the debug build's did. Ticketed
  as #582 (a bug, queued first); Test changes the fixture's site to move within its document after
  it sets the title, which Marley reads.

## Phase 3 — Test
- **Checklist** (no task tool): the fixture's login site ✓; the scenario, a row per REQ ✓; five
  runs, the fifth green ✓; every shot read ✓; 490 and 496 ✓; the golden set with 523 added ✓;
  the gate (below).
- **The login site first.** Chromium sends no event for a title a script sets (#582), so
  `write_login_site`'s `whoami.html` now moves within its document (`history.replaceState`,
  `#read`) after it sets its title, and Marley reads the title there. That is what the release
  install's golden set needed for #507's and #581's scenarios.
- **The scenario.** `script/e2e/523-saved-playwright-scripts.sh`, `compositor sway`, the offline
  Chromium: a `login.html` with a Name field and a Sign in button; a scratch HOME whose `.bashrc`
  puts the box's Node first and points npm at a loopback registry that serves `playwright-core`
  1.63.0, packed from rustal's copy (read only); `log-in.mjs` and `broken.mjs` for the project and
  `where.mjs` for every project, written into the run's config folder.
- **Runs 1 to 4, red, each on the scenario or the code:**
  - Run 1: `RUN_X` missed the Run buttons (they sit at x 1300); the tray, the name field and For
    All Projects were where the guesses put them.
  - Runs 2 and 3: after the first run's terminal splits the window, the tab's pane is the left
    half and its Run buttons move to x 750 (`RUN_X_BESIDE`), measured from the new shot
    `523-02b-installed`.
  - Run 4: `broken` failed in its block, but no recording and no toast came: the watcher read
    the terminal's first block, which is the startup's, finished with 0 before the script began.
    Fixed in `playwright_scripts.rs`: `block_end` finds the block whose command is the typed line
    (F-claude-523, below).
- **Run 5, green:** 15 checks pass. The shots, read:
  - `523-01-tray`: the Scripts button lit left of Pick; the tray under the toolbar: `broken` and
    `log-in` "for this project", `where` "for all projects", each with Run and Edit, then the
    name field, For This Project and For All Projects. REQ-001.
  - `523-02-new`: `check-title.mjs`, made for every project, open in an editor tab with Marley's
    template; the log lists `global/check-title.mjs`. REQ-002.
  - `523-02b-installed`: the window split, the tab and its tray on the left, a terminal on the
    right whose block shows `npm install --prefix …/playwright --no-audit --no-fund &&
    MARLEY_CDP_URL=ws://127.0.0.1:…/devtools/browser/… MARLEY_TAB=… node …/run.mjs …/where.mjs`,
    `added 1 package`, `▶ where on …/login.html`, `tab <id> at …/login.html` and `✓ where
    passed`; the log: `terminal-read` shows the same, the id equal to `browser_tabs`' and
    `node_modules/playwright-core` in the run's data folder. REQ-008 and REQ-005.
  - `523-03-passed`: the tab in front, reading "Signed in as Marley" with the name filled; the
    terminal beside it, `log-in`'s block with `✓ log-in passed`, no install in its command; the
    log: the recordings' count unchanged. REQ-003, REQ-004 and REQ-007.
  - `523-04-failed`: `broken`'s block with its `exit 1` mark and Playwright's timeout, and the
    toast "broken failed with exit code 1; the tab's last minute is recording …"; the log: the
    recording's timeline holds six `script` entries, each run's start and its end (`where` and
    `log-in` with 0, `broken` with 1). REQ-006.
- **Regressions.** 490 and 496, which click the toolbar by coordinates, pass: the button left of
  Pick moved none of their clicks. The golden set with 523 added: 31 of 31, 505 and 518 among
  them (`PICK_X`), and 507 and 581 on the changed login site.
- **Focus.** Every run in a headless sway: "0 Marley windows before the run, 0 after".
- **Noted, not in scope:** opening the new `.mjs` file in an editor tab starts Zed's JavaScript
  language servers, which Zed downloads into the run's profile when they are not there
  (`languages/`), and Zed then says it could not run ESLint. It is Zed's behavior on any
  JavaScript file, and no check rests on it.
- **The gate:** `script/gates.sh --diff`, `GATE GREEN [diff]`, 16 of 16 on the first run.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented.** `CHANGELOG.md` (Added: Playwright scripts kept in Marley);
  `docs/marley/three-prong-plan.md` (the B5c row, shipped);
  `docs/marley_architecture/marley_browser.md` (the recorder's `Script` entry);
  `marley_workbench.md` (Playwright scripts: the tray, the run, the watcher). No path outside the
  Marley-owned set changed; gate:16 found every touchpoint recorded.
- **Knowledge appended:** F-claude-523-a-runs-watcher-read-the-startups-block-001;
  PR-claude-find-a-typed-commands-block-by-its-command-001;
  L-claude-523-chromium-sends-no-event-for-a-scripts-title-001,
  L-claude-523-a-terminal-beside-a-tab-through-zeds-public-calls-001,
  L-claude-523-an-npm-install-in-a-scenario-uses-a-loopback-registry-001;
  AD-claude-523-saved-playwright-scripts-run-in-a-terminal-beside-the-tab-001.
- **Brain:** consultation b96a02155cb74feca776547f7c48a76d closed with
  `decisions/saved-playwright-scripts-run-in-a-terminal-beside-the-browser-tab`.
- **Closed:** TICKET-523 to `tickets/closed/`; its backlog row left at promotion. TICKET-582 (a
  Browser tab's title follows a script's title), found here, is minted and first in the queue.
  The pair is archived to `completed/`.
