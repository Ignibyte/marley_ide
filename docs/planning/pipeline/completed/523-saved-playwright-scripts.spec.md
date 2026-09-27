---
pipeline_id: c8124685-3a11-49ab-b17b-406739d3e88b
ticket: docs/planning/tickets/closed/TICKET-523-saved-playwright-scripts.md
status: Phase 4 — Complete PASS
title: "Saved Playwright scripts, run on a Browser tab"
type: feature
slice: prong 3, after the browser waves; plan D16 (Playwright attaches to Marley's Chromium)
references: [docs/planning/pipeline/completed/499-flight-recorder.spec.md, docs/planning/pipeline/completed/488-browser-pane.spec.md]
---

## Title
Marley keeps Playwright scripts for a project and for every project in its config folder, and
runs one on the Browser tab the user picks: a Marley-owned runner attaches Playwright to the
tab's Chromium through its DevTools endpoint and hands the script that tab's page, while the user
watches the page in the tab and the output in a terminal block beside it. A failed run saves the
tab's last minute as a recording that names the script.

## Scope
### In
- The library: `<config>/playwright/projects/<key>/<name>.mjs` for one project, where `<key>` is
  #507's project key (so a project's worktrees share its scripts, as they share its browser), and
  `<config>/playwright/global/<name>.mjs` for every project; a new script starts from Marley's
  template.
- The runner, embedded in Marley and written to `<data>/playwright/run.mjs` when it differs, with
  `<data>/playwright/package.json` pinning `playwright-core` 1.63.0: it connects with
  `chromium.connectOverCDP(MARLEY_CDP_URL)`, finds the page whose target id is `MARLEY_TAB`, calls
  the script's default export with `{ page, context, browser }`, prints how it ended, and
  disconnects without closing the page or its context.
- `crates/marley_workbench/src/playwright_scripts.rs` (new; named apart from #506's
  `marley_browser::playwright`): listing and creating scripts, writing the runner, building the
  run's command line, starting it in a terminal of the tab's project, and watching that
  terminal's block for its exit code.
- `crates/marley_workbench/src/browser.rs`: a Scripts button in the Browser tab's toolbar, between
  the address bar and Pick, so the buttons after it keep their places, and a Scripts tray under
  the toolbar: each script with its scope, Run and Edit; a name field with
  "For this project" and "For all projects". `marley::PlaywrightScripts` toggles the tray on the
  focused Browser tab. On a failed run, the tab's minute saved through `BrowserHub::record`.
- `crates/marley_browser/src/recorder.rs`: a `Script` entry (the script's name, and at the end its
  exit code), so a recording shows when the script ran.
- `crates/marley_workbench/src/marley_workbench.rs`: the action.
- `script/e2e/523-saved-playwright-scripts.sh`.

### Out (explicitly deferred)
- An MCP tool that runs or writes scripts for agents: AD-claude-492 keeps agents from running
  script in the page. An agent can still write a file into the library, and the user runs it.
- A project's own Playwright (`@playwright/test` in its `node_modules`) and the Playwright Test
  runner (`test()`, `expect`, reporters, traces).
- Scripts kept in the repository (`.marley/playwright/`) or shared through git.
- Turning a recording into a script: #506 drafts Playwright tests from recordings, and its output
  could land in this library later.
- Schedules, arguments per run, and a run history beyond the terminal's scrollback.
- A shell without Marley's integration (fish, #466) gives no block and so no exit code: the run
  still happens, and Marley says it cannot tell how it ended.

## Reference (§20)
N/A for a reference product: Orca runs Playwright only as its own test runner and keeps no
scripts (report 03 §2.15), and Warp has no browser. The mechanism is Playwright's documented
attach to an existing Chromium, `browserType.connectOverCDP`, which the plan chose for this
Chromium on 2026-09-24 (D16: "the Playwright MCP, agent-browser and Claude Code attach to the
endpoint in its `DevToolsActivePort`"). The library follows upstream Zed's own layout for
user-written files: global ones in the config folder, as `paths::snippets_dir()`
(`crates/paths/src/paths.rs:379`) and `tasks_file()` (309) are.

### Prior art
- **Behavior maps and reports.** Report 03 §2.15: no recording, visual diffs or scripts in Orca;
  Playwright is only its end-to-end runner (`tests/playwright.config.ts`). §3 item 9 (for #506):
  the snapshot's role and name map to `page.getByRole`, which the template's example uses.
- **Published material.** Playwright's API, read in `playwright-core` 1.63.0's
  `types/types.d.ts` on the dev box: `connectOverCDP(endpointURL)` takes a CDP WebSocket or HTTP
  URL; the default context is `browser.contexts()[0]`; the connection is "significantly lower
  fidelity" than Playwright's own protocol; `browser.close()` on a connected browser "clears all
  created contexts belonging to this browser and disconnects"; `context.newCDPSession(page)`
  gives a raw session, through which `Target.getTargetInfo` names the page's target id.
  `playwright-core`'s `package.json`: `engines.node >= 20`, 14 MB, no install script, so it
  downloads no browser. The box, read on 2026-09-25: Node 26.9.0 with `npm` and `npx` from mise
  (`~/.local/share/mise/installs/node/26.9.0`); no Playwright that `node` can import (only npx's
  cache holds `playwright-core` 1.63.0, and `~/.local/bin/playwright` is a wrapper that runs
  `mise use -g npm:playwright` first, changing mise's global config); Chromium 152.0.7977.82,
  where Playwright 1.63.0 pins Chromium 153.
- **The code we already ship.** Zed's tasks are saved commands, global (`tasks_file()`) and per
  project (`.zed/tasks.json`, `local_tasks_file_relative_path`, `paths.rs:505`), run in a
  terminal with an environment (`task::SpawnInTerminal`,
  `crates/task/src/task.rs:42`; `TerminalPanel::spawn_task`,
  `crates/terminal_view/src/terminal_panel.rs:632`; `Terminal::wait_for_completed_task`,
  `crates/terminal/src/terminal.rs:3284`). They fit badly here: a task terminal gets no Marley
  shell integration (`terminal.rs:1224`, `task.is_none()`), so its output is no block; no task
  variable names a Browser tab; and per-project tasks live in the repository. The run takes
  Marley's own path instead: a center terminal typed into after the shell's startup handshake, as
  `agents::start_cli` starts an agent CLI (`crates/marley_workbench/src/agents.rs:188`), whose
  command becomes a block with an exit code (`Terminal::blocks`, `terminal.rs:1845`;
  `BlockState` and `ExitCode`, `crates/marley_terminal/src/block.rs:29`, `41`). The endpoint
  comes from `service::endpoint_in` (`crates/marley_browser/src/service.rs:212`). The recorder
  keeps each drawn page's minute and `BrowserHub::record` saves it (`browser.rs:1387`, the path
  Record this takes at 3761). An embedded program written into the data folder is the MCP
  bridge's pattern (`claude_plugin::BRIDGE`, `claude_plugin.rs:28`; `write_bridge_in`,
  `mcp.rs:171`). Does a crate we build own the seam? Zed's `task` owns saved commands but not
  blocks or tabs; Marley's terminal path and recorder own the rest; nothing owns Playwright,
  which stays a Node program outside the process.

## UI proof
UI-AFFECTING (the Browser tab's toolbar and tray, a terminal beside it, a toast).
`script/e2e/523-saved-playwright-scripts.sh` (`compositor sway`: it clicks the tray). Setup: the
offline Chromium; a loopback site whose `login.html` has a name field and a Sign in button that
shows "Signed in as <name>"; a scratch HOME whose `.bashrc` puts the box's Node on the PATH and
points npm at a loopback registry that serves the pinned `playwright-core`, packed from a copy on
the box, so the first run's real install needs no network (setup stops, saying so, when no copy
is found); three scripts: the
project's `log-in.mjs` (fills the field, clicks Sign in, waits for the text) and `broken.mjs`
(clicks a button the page does not have, with a two-second timeout), and the global `where.mjs`
(prints `MARLEY_TAB` and the page's URL). Shots: the tray listing the three with their scopes
(`523-01-tray`); a name typed and "For all projects", the new script open in an editor tab
(`523-02-new`); `log-in` run, the page signed in, at the tab's size, and the terminal beside it
showing the block's output and a success (`523-03-passed`); `broken` run, the block's failure
and the toast naming the recording (`523-04-failed`). The run log carries `where.mjs`'s output
against the stand-in agent's `browser_tabs`, and the saved recording's timeline with its two
`script` entries.

## Locked-In Decisions
- D1: Scripts are files in Marley's config folder, never in a project:
  `<config>/playwright/global/` and `<config>/playwright/projects/<key>/`. They survive branch
  switches, never enter a client's repository, and a project's worktrees share them through #507's
  key (`marley_browser::service::project_key`, landed).
- D2: A script is an ES module whose default export is an async function of
  `{ page, context, browser }`; a throw or a rejected promise fails the run. Scripts never import
  Playwright themselves (nothing is installed beside them for Node to find). `MARLEY_CDP_URL` and
  `MARLEY_TAB` are in the run's environment: the runner reads them, and a script passes them on to
  any tool it starts, such as a project's own Playwright Test or a Python Playwright script.
- D3: Marley's own Playwright: `playwright-core` pinned to 1.63.0 in `<data>/playwright`, installed
  by `npm install` in the run's own block the first time, so the user sees the download. It
  brings no browser; the script drives the tab's Chromium. Node is the terminal's own (`node` on
  its PATH, 20 or newer).
- D4: Running is the user's act, from the tray or its action, never an agent's tool call
  (AD-claude-492).
- D5: A run is one command typed into a new terminal of the tab's project, in a pane split beside
  the tab's pane, the tab staying in front: `MARLEY_CDP_URL=<ws URL> MARLEY_TAB=<target id> node
  <data>/playwright/run.mjs <script path>`, with the install prefixed on the first run. The block
  shows exactly what ran, and the shell's history can run it again. Zed's task terminals are not
  used: they have no blocks (see Prior art).
- D6: A block that ends with a non-zero exit code saves the tab's last minute as a recording,
  with a `Script` entry where the run started and another where it ended (name and exit code),
  and a toast names the recording. A run that passes saves nothing. The recording holds what
  Marley sees of the page (frames, console, requests, navigations, snapshots); Playwright's own
  clicks are not Marley's input, so they show in the frames rather than as click entries.
- D7: The runner never closes the page, the default context or the browser: at the end it only
  disconnects, so the tab and its page outlive every run.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens a Browser tab's Scripts tray, the system shall list the scripts saved for the tab's project and those saved for every project, each marked with its scope. | Shot `523-01-tray` |
| REQ-002 | WHEN the user saves a new name in the tray for the project or for every project, the system shall create `<name>.mjs` from Marley's template in that library and open it in an editor tab. | Shot `523-02-new`; the log lists the library |
| REQ-003 | WHEN the user runs a script, the system shall open a terminal of the tab's project in a pane beside the tab and run the script there as one command, whose block shows its output and exit code. | Shot `523-03-passed` |
| REQ-004 | WHILE a script runs, the system shall keep the tab it was run from in front and drawing at the tab's size, and give the script that tab's page to act on. | Shot `523-03-passed` (the page signed in) |
| REQ-005 | WHEN the system runs a script, it shall set `MARLEY_CDP_URL` to the WebSocket endpoint of the tab's Chromium and `MARLEY_TAB` to the tab's target id in the run's environment. | The run log: `where.mjs`'s output against `browser_tabs` |
| REQ-006 | WHEN a script's block ends with a non-zero exit code, the system shall save the tab's last minute as a recording with the script's start and end in its timeline, and name the recording in a toast. | Shot `523-04-failed`; the log: the recording's timeline |
| REQ-007 | WHEN a script's block ends with exit code 0, the system shall save no recording. | The log: `browser_recordings` before and after `log-in` |
| REQ-008 | WHEN a run finds no `playwright-core` in Marley's data folder, the system shall install the pinned version there, in the run's own block, before the script runs. | The run log: the first run's block (`terminal-read`), npm's install from the loopback registry, then the script's output |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. Done at promotion: #507's
  key is there, and a scratch Chromium showed `connectOverCDP` leaving the page's viewport and
  the page itself alone.
- **P2 Code:** the library, the runner and its template, the tray, the run and its watcher, the
  recorder's entry; fmt and clippy clean; a review of the diff against each REQ and against
  AD-claude-492.
- **P3 Test:** write and run the scenario and read every shot; rerun the scenarios that click the
  toolbar by coordinates (490 and 496; 505 and 518 run in the golden set), which the new button,
  left of Pick, must not move (L-claude-498); the golden set with 523 added; `script/gates.sh
  --diff` green.
- **P4 Complete:** CHANGELOG; the plan's prong 3 section; `docs/marley_architecture/marley_browser.md`
  (the recorder's new entry); the ledger capture; close the ticket, archive, commit.
