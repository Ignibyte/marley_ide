---
pipeline_id: b1ecc9ef-37e6-4e30-a7fb-8acdc64e64f7
ticket: docs/planning/tickets/open/TICKET-945-system-pages-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The system pages as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/G-system.md, docs/marley/look-plan.md]
---

## Title
The System tab's pages say what the SystemPages board says: Install and update lists the last
install's steps and warns when a newer build is installed while this one runs; Marley's MCP endpoint
shows its bridge, tool count, last refusal and limits, with Open Agent Activity and Copy the
endpoint; Marley's Chromium shows its version, the sites each profile keeps cookies for, and its
limits; Shell integration, Fleet providers and For developers get their buttons.

## Scope
### In
- `script/install-marley` (Marley-owned): at the end of a full install, on Linux and on macOS,
  writes `install.json` into Marley's data folder (`${XDG_DATA_HOME:-~/.local/share}/marley`, or
  `~/Library/Application Support/Marley`): the commit, whether the tree had changes, when it ended,
  the installed binary's path, and one entry per step (the toolchain, with `rustc --version`; the
  build, with how long it took; the remote server, with its platform; the desktop entry, or the
  app on macOS), each with its result. Written to a temporary name and renamed into place.
  `--remote-server-only` writes nothing.
- `crates/marley_workbench/src/system_pages.rs` (Marley crate):
  - Install and update: reads `install.json` with `read_install_in(data_dir)`; one row per step
    with its mark and value; then, when the running binary is the installed one and its commit
    differs from the record's, a warning row "Running Marley · still on <short commit> · restart".
    With no record, the rows #885 shows stay. The button reads "Open the log" (Marley's log, as
    `workspace::OpenLog`).
  - Marley's MCP endpoint: Endpoint (its address, "loopback only", up or stopped), Bridge (the
    bridge's path, "stdio"), Tools ("N tools · M write classes granted"), Last refusal (tool ·
    code · what it acted on, and when; "none today" without one), Limits ("request bodies ≤ 1 MiB
    · loopback only"); Paired programs stays. Buttons: Open Agent Activity, Copy the endpoint.
  - Marley's Chromium: a Chromium row (the version a running browser reports, "running" or "not
    started"), then one row per project's profile (its pages, and "cookies from N sites" while its
    browser runs), then a limits row from the guide's "Limits of the Browser tab".
  - Buttons: Shell integration, Open the guide; Fleet providers, Add a provider (opens the
    user's settings file) and Open Settings (the Marley page); For developers, Open the plan (the
    public plan on GitHub) and Open the guide.
- `crates/marley_workbench/src/browser.rs` (Marley crate): a `pub(crate)` reader of the version a
  project's browser reported (from its `Identity`), and a call counting the cookie domains of a
  running project's browser (`Storage.getCookies`).
- `crates/marley_browser/src/page.rs` (Marley crate): `Identity::product()`, the browser's name
  and major version from its user agent.
- `crates/marley_mcp/src/transport.rs` and `crates/marley_mcp/src/marley_mcp.rs` (Marley crate):
  `MAX_BODY_BYTES` made public and re-exported, so the page names the limit the server enforces.
- `crates/marley_workbench/src/clients.rs`, `mcp.rs`, `agent_activity.rs` (Marley crate):
  `pub(crate)` readers for the bridge's path, the served tool count, the granted write classes and
  today's last refusal.

### Out (explicitly deferred)
- Restart Marley: #885 Out ("an action that ends … wants its own ticket and its confirmation").
- Clear terminal history: #855 and #885 Out (same reason).
- Rusty's version on Troubleshooting: #885 Out (kept private in Rusty's state).
- Chromium's "Clear a project's data…" button: the rail's per-project Clear Browser Data… (AD-581)
  covers it; a page button needs a project picker first. A follow-up ticket.
- This build's lead sentence and the Data folder's "shared" value; Fleet providers' addresses;
  For developers' planned items with their status and the Gate and Visual checks rows: look gaps
  the review lists, for a follow-up ticket.
- A toast when a newer build is installed: the page's row is the board's notice; a toast is a
  separate decision.

## Reference (§20)
N/A — Marley-specific: the System tab is Marley's own (#855, #885), and every row reports a fact of
Marley's install, MCP server (`marley_mcp`) or browser service (`marley_browser`). Chromium's
version and cookies come over CDP (`Browser.getVersion`, `Storage.getCookies`), per the published
DevTools protocol.

### Prior art
- `crates/marley_workbench/src/system_pages.rs:266` `install_rows` (fixed sentences); `:429`
  `mcp_rows` (Server, address file, paired programs); `:465` `chromium_rows` (per project: profile
  path and pages); `:393` `shell_rows`; `:509` `fleet_rows`; `:553` `developer_rows`; `:754`
  `render_buttons` (Shell, MCP, Chromium, Fleet and Developers have none); `:149` `Row` and `:141`
  `Mark`.
- `script/install-marley`: the Linux flow ends at `:490`–`:511` (installs, desktop entry, prints
  the commit and whether Marley is running); the macOS flow writes `Marley.app` (`:204`); the
  binary goes to `$lib/marley` by a rename (AD-claude-502), so a running Marley keeps its old file.
- `crates/marley_workbench/src/mcp.rs:108` the grant table, ten write classes; `:353`
  `write_bridge_in`, the bridge at `<data>/mcp/marley-mcp-bridge`; `crates/marley_mcp/src/registry.rs:913`
  `tools_list`; `crates/marley_mcp/src/transport.rs:673` `MAX_BODY_BYTES` (1 MiB, private);
  `crates/marley_mcp/src/auth.rs` `is_loopback`.
- `crates/marley_workbench/src/agent_activity.rs:72` `ActivityRow` with `Outcome::Refused { code }`
  (`:88`); `AgentActivity` keeps today's rows; `OpenAgentActivity` opens the tab.
- `crates/marley_workbench/src/browser.rs:725` each project's `identity`, read with
  `Identity::read` (`Browser.getVersion`) at connect (`:3870`); `crates/marley_browser/src/page.rs:135`
  `Identity`.
- `docs/marley/guide.md:3273` "Limits of the Browser tab".
- `crates/marley_workbench/src/guide.rs` `OpenGuide`; `OpenSettings` (`marley_workbench.rs:1253`);
  Zed's `zed_actions::OpenSettingsFile`.
- AD-claude-581: clearing a project's browser data asks first and deletes its profile; the rail
  offers it.
- AD-claude-703: the agent activity log and kill switch; a stopped switch refuses every write tool
  with its own code, which gives the scenario a refusal to show.
- L-claude-871: a scenario's MCP client that acts runs in Marley's terminal with that terminal's
  token.

## UI proof
Scenario `script/e2e/945-system-pages-as-the-board.sh` (sway). Setup: a scratch repository with a
small HTML page that sets a cookie, served on loopback by the scenario; `install.json` written into
the run's data folder naming the debug binary and another commit; #871's MCP client in the
`.bashrc` of Marley's terminal (stand-ins first on the terminals' PATH).
- `945-01-install`: Install and update: the four steps and "Running Marley · still on …"
  (REQ-001, REQ-002).
- `945-02-mcp`: agent control stopped, the client's `terminal_run` refused, then the MCP page:
  Endpoint, Bridge, Tools, Last refusal, Limits, the two buttons (REQ-003, REQ-004).
- `945-03-activity`: Open Agent Activity clicked: the Agent Activity tab (REQ-004).
- `945-04-chromium`: the page opened in a Browser tab, then the Chromium page: the version, the
  project's profile with "cookies from 1 site", the limits row (REQ-005).
- `945-05-buttons`: Shell integration, Fleet providers and For developers in turn, each with its
  buttons (REQ-006).
- `945-06-day`: Marley Day, Install and update (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — The install writes what it did; the page reads it. Marley never runs or times an install
  itself, and a missing or unreadable record leaves #885's rows.
- D2 — The newer-build row shows only when the running binary is the installed one (its path,
  with Linux's " (deleted)" suffix taken off, is the record's) and the commits differ, so a debug
  build never claims to be stale.
- D3 — "Signed in" cannot be known; the profile row counts the sites its browser holds cookies
  for and says so ("cookies from N sites"). A browser not running shows its pages only.
- D4 — Every number on the MCP page comes from what the server enforces or serves: the tool list,
  the grant table, `MAX_BODY_BYTES`, the activity log. Nothing is typed in by hand.
- D5 — Add a provider opens the user's settings file, where `marley.fleet.providers` lives; there
  is no provider form yet.
- D6 — Open the plan opens `docs/marley/three-prong-plan.md` on the public repository in the
  system browser; an installed Marley has no checkout path to open.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Marley's data folder holds an install record, Install and update shall list the toolchain, the build with its time, the remote server and the desktop entry, each with its result, in Marley Night and Marley Day. | `945-01-install`, `945-06-day` |
| REQ-002 | WHILE the running binary is the installed one and the installed commit differs from the running one, Install and update shall show "Running Marley · still on <commit> · restart" as a warning. | `945-01-install` |
| REQ-003 | The MCP endpoint page shall show the endpoint, the bridge, the tool count with the granted write classes, the last refusal and the limits. | `945-02-mcp` |
| REQ-004 | The MCP endpoint page shall offer Open Agent Activity, which opens the Agent Activity tab, and Copy the endpoint. | `945-02-mcp`, `945-03-activity` |
| REQ-005 | The Chromium page shall show the browser's version, each profile's pages and the sites it keeps cookies for while it runs, and the limits of the Browser tab. | `945-04-chromium` |
| REQ-006 | Shell integration shall offer Open the guide; Fleet providers, Add a provider and Open Settings; For developers, Open the plan and Open the guide. | `945-05-buttons` |
| REQ-007 | WHEN `script/install-marley` finishes a full install, it shall write the install record, and WHEN it runs with `--remote-server-only`, it shall write none. | the review of the diff, and `945-01-install`'s record format |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the install record, the readers, the four pages' rows and buttons; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
