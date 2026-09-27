# Claude Code's events, slice 2: the plugin update, fleet_snapshot, and stale rows — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-547-claude-code-events-slice-2.md
- **Pipeline spec:** 547-claude-code-events-slice-2.spec.md

## Phase 1 — Plan
- **Request:** #519's Plan cut its design in two (2026-09-26); this is the second half: D9's chip,
  item 7 (the MCP snapshot) and D7 (the stale form). Chad's standing goal of 2026-09-25 covers
  it ("The rest go ahead and begin implementing it now").
- **Classification / tier:** feature, prong 2 (C1's first piece, slice 2). Marley crates, plus
  three small additive Zed touches for the setting. Size M.
- **Checklist (no TaskCreate in this harness):** pick · pre-flight · recall · mint · prior art ·
  spec · design · present: all done below.
- **Pre-flight:** gate, e2e runner and hooks OK; no active pipeline; the README marker present;
  cargo busy with `just install`'s release build, so this phase ran no cargo at all.
- **Recall (§18.3):**
  - AD-claude-482 (the plugin through Claude Code's own `claude plugin` commands, from a local
    marketplace Marley writes): the update follows the same path (D3).
  - L-claude-482 (background work in a Marley crate is a lazy future, not an `async` block that
    never awaits): the update's file writes use `futures::future::lazy`, as `run_install` does.
  - AD-claude-fleet-rail-quiet-no-transport-until-brain-001: nothing has fed the server's
    `FleetSnapshot` since the forge client left; `ServerData.snapshot` is its default.
  - #519's completed notes: the seats, their ids, the labels, and what slice 1 left for here.
  - Brain: consultation 3bd3ea0a7f9a492bb24a3360bdecc2c7, nothing on this seam (a narrower
    repeat, 4a697073…, was closed with no decision).
- **Discovery (re-verified at `ca70b6488d`):**
  - `crates/marley_workbench/src/mcp.rs:51-55` (`McpServer { failure }`), `:62-120` (`start`:
    the `Shared` is built and moved into `transport::spawn`, then dropped from scope).
  - `crates/marley_mcp/src/transport.rs:34-46` (`ServerData.snapshot`, `version`), `:369`
    (`signal_change`); `crates/marley_mcp/src/dispatch.rs:159-163` (`Family::Fleet` answers
    `tools::fleet_snapshot_result(ctx.snapshot)`); `resource.rs:13` (`fleet://snapshot`).
  - `crates/marley_workbench/src/claude_plugin.rs:69-80` (`ClaudePlugin`), `:103-122`
    (`set_up`), `:145-149` (`installed_in`), `:174-241` (`install`, `run_install`);
    `crates/marley_workbench/src/agent_bar.rs:200` (where the chip is placed), `:305-340`
    (`claude_plugin_chip`, shown only while `installed == Some(false)`).
  - `crates/marley_workbench/src/agent_events.rs` (`AgentEvents`, `on_frame`, `forget`);
    `crates/marley_workbench/src/rail.rs` (`refresh`, `terminal_snapshot`, `quiet_timers`);
    `crates/marley_agent/src/claude_events.rs` (`seat_line`).
  - `crates/settings_content/src/marley.rs` (`MarleySettingsContent`),
    `crates/settings_ui/src/marley_page.rs` (`agents_section`, an array of two items),
    `assets/settings/default.json:1664-1670` (the `marley` block);
    `crates/marley_workbench/src/marley_workbench.rs` (`MarleySettings::from_settings`).
    `settings_ui` renders `u64` fields (`add_basic_renderer::<u64>`).
  - `script/e2e/515-marley-settings-page.sh` clicks the Layout dropdown, which is above the
    Agents section, so a new Agents item moves nothing it clicks.
  - Claude Code 2.1.283: `claude plugin update [options] <plugin>`, `claude plugin marketplace
    update [name]`; `installed_plugins.json` format 2, entries with `scope` and `version`.
- **Decisions:** D1 to D7 in the spec.

### Design
- **Approach.**
  1. **The setting.** `MarleySettingsContent::no_update_after_minutes: Option<u64>` (Zed's
     `settings_content`); `default.json`'s `marley` block sets 30; `MarleySettings` gains
     `no_update_after_minutes: u64` (30 when absent; 0 turns the form off); the Marley page's
     Agents section gains "No Update After Minutes" (`agents_section` returns three items).
  2. **The chip and the update** (`claude_plugin.rs`, `agent_bar.rs`). `ClaudePlugin` gains
     `installed_version: Option<String>` and `updating: bool`; `set_up` reads the version with
     `installed_version_in(config_dir)` (the `marley@marley` entry of scope `user`, else its
     first). `shipped_version()` parses the embedded `plugin.json`'s `version` with `semver`;
     `ClaudePlugin::needs_update` is true while the plugin is installed and its version parses
     and is older. `update(plugin, workspace, cx)` mirrors `install`: off the main thread it
     rewrites the marketplace (`write_plugin_in`), then runs `claude plugin marketplace update
     marley` (or `marketplace add <dir>` when Claude Code no longer knows it) and `claude plugin
     update marley@marley` through `crate::run_program`; success sets the installed version to
     the shipped one and shows a toast, failure shows the error. The chip reads "Update Marley's
     plugin" while `needs_update`, and "Updating Marley's plugin for Claude Code…" while it runs.
  3. **Publishing** (`mcp.rs`, `agent_events.rs`). `McpServer` keeps `shared:
     Option<transport::Shared>` (a clone taken before `spawn`; `None` when it failed).
     `start` observes `AgentEvents`; `publish` copies the app's snapshot into
     `ServerData.snapshot` under the lock and calls `transport::signal_change`.
     `AgentEvents::snapshot()` exposes the snapshot, and `AgentEvents::any_working()` says
     whether a seat works.
  4. **The seat's end** (`agent_events.rs`, `rail.rs`). `agent_events::end(views, cx)` applies
     `Ended` to each live seat named. `Rail::refresh`, after building its snapshot, names the
     listed terminals that have a live seat and no Claude Code in the foreground; ending them
     notifies the global, and the next refresh finds nothing to end.
  5. **The stale form** (`claude_events.rs`, `rail.rs`). `seat_line(seat, now_ms,
     no_update_after_ms)`: a working seat whose last event is at least the threshold old reads
     `no update in N m`, N the whole minutes since it. `terminal_snapshot` passes the epoch time
     and the setting. `Rail` keeps `minute_timer: Option<Task<()>>`: while `any_working`, a
     one-shot 60 s timer that refreshes and is re-armed by that refresh.
  6. **The stand-in agent** (`script/e2e/browser-fixture.sh`): `fleet` calls `fleet_snapshot`
     and prints each seat's id, state and the labels a row shows, or `no seats`.
- **File manifest.**
  - Marley: `crates/marley_workbench/src/claude_plugin.rs`, `crates/marley_workbench/src/agent_bar.rs`,
    `crates/marley_workbench/src/mcp.rs`, `crates/marley_workbench/src/agent_events.rs`,
    `crates/marley_workbench/src/rail.rs`, `crates/marley_workbench/src/marley_workbench.rs`,
    `crates/marley_workbench/Cargo.toml` (`semver`), `crates/marley_agent/src/claude_events.rs`,
    `script/e2e/browser-fixture.sh`, `script/e2e/547-claude-code-events-slice-2.sh` (Test),
    `script/e2e/golden`.
  - Zed: `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
    `assets/settings/default.json`.
  - Generated: `Cargo.lock`.
- **Ledger rows.** The three Zed paths' rows in `docs/marley/zed-touchpoints.md` gain
  `no_update_after_minutes` (#547) before their edits.

### E2E plan
Fixtures: a scratch repository; a HOME whose `.bashrc` puts `$E2E_WORK/bin` first on the PATH,
which setup also exports for Marley, so `which claude` finds the stand-in; the stand-in `claude`
of #519 with a `plugin` branch (log the arguments; for `plugin update marley@marley`, write
1.2.0 into the scratch `installed_plugins.json`); `CLAUDE_CONFIG_DIR` exported to a scratch
folder whose `plugins/installed_plugins.json` lists `marley@marley` 1.1.0 and whose
`known_marketplaces.json` lists `marley`; the profile's `marley` settings block given
`"no_update_after_minutes": 1`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | type `claude`, wait 3 s | `547-01-update-chip` |
| REQ-002 | click the chip, wait 3 s | `547-02-updated`; `holds` on the plugin log: `plugin marketplace update marley`, then `plugin update marley@marley` |
| REQ-003 | Enter: UserPromptSubmit, PreToolUse (Bash); `mcp_agent fleet` | the run log: the seat `working` with the prompt |
| REQ-005 | wait 70 s | `547-03-no-update` |
| REQ-003 | Enter: StopFailure (`rate_limit`); `mcp_agent fleet` | the run log: `error`, `rate_limit` |
| REQ-004 | Ctrl-D (the stand-in exits); `mcp_agent fleet`; close the terminal from the palette; `mcp_agent fleet` | the run log: `done`, then `no seats` |
| REQ-006 | the golden set's 515 run | its `515-01-marley-page` shot shows the Agents item |
| REQ-007 | `just regress` with 547 in the set; `script/gates.sh --diff` | their output |

Not reached: a real Claude Code's update (the stand-in stands for `claude plugin`; the commands'
names and arguments are the ones 2.1.283's `--help` lists), and 30 real minutes (the setting
stands in).

### Risks
- The minute timer refreshes each rail once a minute while any seat works anywhere: cheap, and it
  stops with the last working seat.
- A misread foreground (the cache refreshes on output) could end a live seat; its next frame moves
  it out of `done` again, since the fold starts from the seat's state.
- A user's own marketplace named `marley` would be updated in place of Marley's: the same
  exposure `install` has had since #482.
- `claude plugin update` says a restart applies it; the toast says so, rather than #482's
  `/reload-plugins`.

## Phase 2 — Code
- **Built.**
  - The setting: `MarleySettingsContent::no_update_after_minutes` (Zed's `settings_content`),
    30 in `default.json`'s `marley` block, `MarleySettings::no_update_after_minutes` (30 when
    absent, 0 off), and "No Update After Minutes" in the Marley page's Agents section.
  - The chip and the update: `ClaudePlugin::installed_version` and `updating`,
    `installed_version_in`, `shipped_version` (the embedded manifest, now one `MANIFEST`
    constant that `FILES` ships too), `ClaudePlugin::needs_update` (`semver`), `update` and
    `run_update`; `write_marketplace` now holds the half `run_install` and `run_update` share.
    The chip offers "Update Marley's plugin" while `needs_update`.
  - Publishing: `McpServer::shared`, a clone of the transport's data taken before `spawn`;
    `mcp::publish`, an observer of `AgentEvents`; `AgentEvents::snapshot` and `any_working`.
  - The seat's end: `agent_events::end`, called by `Rail::note_claude_code` after each refresh
    for the listed terminals without Claude Code in the foreground.
  - The stale form: `claude_events::seat_line(seat, now_ms, no_update_after_ms)` through
    `marley_fleet::is_stale`; `terminal_snapshot` reads the epoch time and the setting; the
    rail's `minute_timer`.
  - The stand-in agent's `fleet` command; the scenario (for Test).
- **Deviations.**
  - `end` leaves a failed seat as it is, not only an ended one. The reducer keeps an `Error`
    seat in `error` on `Ended`, so a failed seat would never read as ended: each refresh would
    end it again, notify the global and refresh again, a loop. A failed seat stays failed until
    its terminal closes or a new session starts; the scenario proves `done` through an idle seat.
  - `terminal_snapshot` reads the time and the setting itself rather than taking them as
    arguments: it already takes seven, clippy's limit.
- **Review of the diff.**
  - Re-entrancy: `end` and `publish` touch only globals; the rail's own refresh calls `end`,
    whose notification comes back to the rail at the next effect flush, when it finds nothing
    to end. The minute timer's task sets `minute_timer` to `None` before refreshing, so the
    refresh can arm the next one.
  - The update's failure reaches the workspace (`show_error`) and leaves the chip offered.
  - Upstream: three additive Zed hunks, each with its row widened first.
- **Checks.** `cargo check --all-targets`, `cargo clippy --all-targets --all-features -D
  warnings` (after two findings: `update`'s first doc paragraph too long, and
  `note_claude_code`'s window taken as `&mut` where `&` does) and `cargo fmt --check` over
  `marley_agent`, `marley_workbench`, `settings_content` and `settings_ui`: clean.

## Phase 3 — Test
- **The scenario:** `script/e2e/547-claude-code-events-slice-2.sh` (`compositor sway`), as planned,
  with one change: the stand-in `claude` is named by `MARLEY_CLAUDE`, not by the PATH (below).
- **Reds, and what changed.**
  - The first run's click ran the REAL `claude`: Marley's PATH, as the app sees it, put
    `~/.local/bin` ahead of the scenario's folder, so `which claude` found Claude Code itself,
    which ran `plugin marketplace update marley` against the scratch `CLAUDE_CONFIG_DIR` and
    refused its minimal `known_marketplaces.json` (the error showed in the workspace, as REQ-002
    wants of a failure). The scratch config kept it off Chad's: his `known_marketplaces.json` and
    `installed_plugins.json` are unchanged since 2026-09-25 19:22. Fixed: `claude_plugin::init`
    runs the `claude` that `MARLEY_CLAUDE` names, as `MARLEY_CHROMIUM` names Chromium, and the
    scenario's `known_marketplaces.json` now has Claude Code's own shape.
  - `547-03-no-update` first read `working · Run the test suite` after 70 quiet seconds. The
    minute timer was armed at the turn's first frame (the prompt), a few milliseconds before its
    last (the tool call), so when it fired the seat had been quiet just under a minute; it
    re-armed for another minute and the shot came first. Fixed: `AgentEvents::next_quiet_change`
    gives the delay to the moment a row next changes (the threshold after the last event, then
    each whole minute), and the rail arms its timer for that, plus 100 ms.
  - The gate's dylint `blocking_io_on_foreground` flagged `publish` locking the server's data on
    the main thread. Fixed: `publish` sends the snapshot down a channel to one background task
    that locks, replaces and signals, in order (`mcp::publisher`).
- **The shots, read:**
  - `547-01-update-chip`: the agent bar reads "Update Marley's plugin" with its arrow icon, the
    row `Claude Code · waiting` from the quiet timer (REQ-001).
  - `547-02-updated`: the toast "Marley's plugin for Claude Code is updated, and now tells the
    rail what Claude Code is doing. New Claude Code sessions use it; restart a running one to
    pick it up." and no chip (REQ-002).
  - `547-03-no-update`: `no update in 1 m · Run the test…`, then `Bash: cargo test` (REQ-005).
  - The golden run's `515-01-marley-page`: the Agents section's "No Update After Minutes" with
    its number field at 30 (REQ-006; `default.json` sets 30).
- **The checks** (the final run): the plugin log holds `plugin marketplace update marley` then
  `plugin update marley@marley` (REQ-002); `fleet` gives the seat `working` with the prompt and
  `Bash: cargo test`, then `idle` with the last message (REQ-003), then `done` after Ctrl-D, then
  `no seats` after `exit` closed the terminal (REQ-004). Sway: the user's Hyprland untouched.
- **Golden set:** 547 joins it; `just regress` on the debug build: all 12 pass (547 in 104 s).
- **Gate:** `script/gates.sh --diff`: red once on dylint (above), then `GATE GREEN [diff]`, 16
  passed.
- **Not reached:** a real Claude Code's `plugin update` (the stand-in answers for it; the
  commands are the ones 2.1.283's `--help` lists), and 30 real minutes (the setting at 1).
- **Verdict:** Phase 3 PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `docs/marley/three-prong-plan.md` (C1's row);
  `docs/marley_architecture/marley_workbench.md` (the seat's end, `next_quiet_change`,
  publishing, the update and `MARLEY_CLAUDE`) and `marley_agent.md` (`seat_line`'s stale form);
  the three Zed rows in `docs/marley/zed-touchpoints.md` describe what shipped.
- **Knowledge:** F-claude-547-a-scenarios-click-ran-the-real-claude-001,
  F-claude-547-a-timer-armed-at-the-first-event-fired-before-the-last-was-a-minute-old-001,
  PR-claude-name-the-fakes-the-app-runs-001, L-claude-547-marleys-path-is-the-login-shells-001.
- **Brain:** consultation 3bd3ea0a7f9a492bb24a3360bdecc2c7 closed with
  `decisions/marley-fleet-snapshot-serves-the-apps-claude-code-seats-the-plugin-updates-from-the-chip`
  (follow-up by 2026-10-10).
- **Ticket:** closed.
