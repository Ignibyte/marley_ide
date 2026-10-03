---
pipeline_id: 1d2b658c-59eb-4ed1-b9c5-866af3a5a546
ticket: docs/planning/tickets/open/TICKET-643-rusty-switch-and-connection.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Marley connects to Rusty when Rusty is turned on"
type: feature
slice: Rusty in Marley R1 (rusty-in-marley.md R-D0, R-D1, R-D2, R-D8); prong 2 C2
references: [docs/marley/rusty-in-marley.md, docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/633-rusty-tools-for-zeds-agents.spec.md, docs/planning/pipeline/completed/632-the-embedded-harness.spec.md]
---

## Title
`marley.rusty` is Rusty's one switch in Marley, off by default. On, Marley starts `rusty-mcp` on
stdio (`connection: embedded`) or connects to Rusty's running service (`service`), keeps that
connection, and shows its state on a Rusty section of the Settings window's Marley page, with
Rusty's own embedding provider read and written through Rusty's tools. `marley.rusty_tools` (#633)
moves into the block as `agent_tools`, carried over by a settings migration. The ticket also
creates `crates/marley_rusty`, the pure core the Rusty-in-Marley slices build on. Chad, 2026-10-02:
"The mcp then lives inside of the marley ide but enabled/disabled"; on this batch, 2026-10-03:
"lets make a plan to begin the work and spec out the tickets", then "Queue all five".

## Scope
### In
- **The block:** `marley.rusty: { enabled, connection, service_url, agent_tools }` in
  `settings_content` (`MarleyRustySettingsContent`; `MarleyRustyConnection { Embedded, Service }`
  with the strum derives the page's dropdowns take), in `default.json` as `enabled: false`,
  `connection: "embedded"`, `service_url: "http://127.0.0.1:4174/mcp"` and `agent_tools: false`,
  resolved as `MarleySettings::rusty`, each switch read "off unless on" (D1).
- **The carry-over:** `marley.rusty_tools` leaves `MarleySettingsContent`, `default.json` and the
  Agents section; a settings migration in Zed's `migrator`, in a module of Marley's own, moves a
  user's value into the block at each load (D2).
- **Finding `rusty-mcp`:** `MARLEY_RUSTY_MCP`, else the search path, at each connect, off the main
  thread; #633's offer finds it the same way (D3).
- **The connection:** `marley_workbench::rusty` keeps one client, Zed's `ContextServer::stdio` for
  `embedded` (`rusty-mcp` with no arguments, as Rusty's `.mcp.json` names it) or
  `ContextServer::http` for `service`; checks it with `ping`; says why it is down; connects again
  after 1, 2, 4 … 60 s; drops it, and with it the child, when the switch turns off or the source
  changes, while Marley runs (D4, D10).
- **Live refresh (R-D2):** the embedded connection reads Rusty's settings again on
  `notifications/resources/list_changed`; the service connection reads them on connect and after
  each write (D5).
- **Rusty's settings:** `settings_list` read into a typed view; `embedding_provider` shown as
  Rusty's four values and written with `setting_set`, then read again (D6).
- **The Settings window:** a Rusty section on the Marley page after System One, with Rusty,
  Connection, Service URL, Rusty Tools for Agents (moved from the Agents section) and a link to a
  Rusty's Server sub-page that draws the connection's state and Rusty's settings: a view
  `marley_workbench::rusty` registers with the settings UI (D7).
- **Rusty's tools for Zed's agents** (#633) are offered only while `enabled` and `agent_tools` are
  both on.
- **`crates/marley_rusty`** (new; `[lib] path = "src/marley_rusty.rs"`, `MIT OR Apache-2.0`, no
  gpui): the `settings_list` view and `EmbeddingProvider`, the names of the tools it reads,
  `fixtures/` in `rusty-mcp`'s answer shape, and the stand-in `rusty-mcp` for scenarios (D8).
- **The e2e harness:** `script/e2e.sh` turns Rusty off in each run's copy of the user's settings,
  points its service at nothing and names no `rusty-mcp`; 633's scenario, and 642's once it lands,
  move to the new keys and the shared stand-in (D9).
- `crates/zed/src/zed.rs`: the tests' `marley.rusty_tools = Some(false)` (#634) becomes Rusty off.
- `script/e2e/643-rusty-switch-and-connection.sh`.

### Out (explicitly deferred)
- **What #644 to #647 add**, each to `marley_rusty` and `marley_workbench::rusty`: the rail's Brain
  view (#644: the `brain_tree` view and the tree's port of Ely's FileTree, plus the stand-in's
  fixture vault and its brain-tool answers, since it is the first ticket to read the vault); the
  Page tab (#645: `brain_render`'s fields and the markdown pass with wikilinks); the Knowledge
  panel (#646: the `brain_get_links` and `brain_search` views); the Graph tab (#647: the
  `brain_graph` view and the force layout). This ticket ships no panel, tab, dock button or rail
  switch, so R-D0's "no panel" holds by having none.
- Rusty's other server settings (`embedding_model`, `ollama_url`, `pin_timeout_minutes`, the skill
  keys) and the provider in use from `brain_semantic_status`: slice R8.
- A change cursor (`changes_since`, Rusty's TICKET-035, not built): until it ships, a setting
  another process writes shows at the next connect, `list_changed` or write, and service mode
  polls nothing.
- Rusty's structured render (TICKET-036) and bookmarks (TICKET-037): nothing here reads them.
- Offering Rusty's service to Zed's agents over HTTP: the offer stays #633's stdio `rusty-mcp`, so
  with `agent_tools` on and `connection: embedded` two `rusty-mcp` run, Marley's and the agents'.
- Writing the migrated `settings.json`: Zed's banner offers it; Marley writes nothing.
- Ely GPUI Components: nothing ported (D7); R-D10's settings story waits for R8's rows.
- The Settings window's search over the sub-page's content; a `service_url` off this machine.

## Reference (§20)
Upstream Zed, kept as it is: the Settings window's sub-pages (`settings_ui`'s `SubPageLink` with a
render function, as the MCP Servers and Skills pages are), the status row each context server gets
there (`ui::AiSettingItem` in `mcp_servers_page.rs`), Zed's MCP client (`context_server`'s
`ContextServer::stdio` and `http`, `ping`, `on_notification`), and Zed's settings migrations
(`migrator::migrate_settings`, run in memory at each load by `SettingsStore`, with the banner over
`settings.json` that offers to write the file). Marley adds a section, a sub-page and one
migration. What the connection does is Marley-specific: Rusty is Ignibyte's own store, and
neither Warp nor Zed connects to one; the switch and the supervision follow Marley's own
`marley.system_one` (#565) and embedded harness (#632). No Warp code.

### Prior art
- **Behavior maps:** `docs/warp_architecture/crates/mcp.md` (Warp's MCP client: a handle per
  connected server, listing that fails soft, errors worded for the user; read for the status
  line's duty to say why, no code). `docs/zed_architecture/` maps none of `settings_ui`,
  `migrator` or `context_server` (checked `crates/` and `subsystems/`), so their source is the map.
  `docs/orca_architecture/06-cli-automations-skills.md` §2.15: Orca has no MCP server or client to
  copy. The plan `docs/marley/rusty-in-marley.md` (R-D0 to R-D2, R-D8, R-D10) and Rusty's triage.
- **Published material:** the MCP specification's lifecycle (`initialize`, the `ping` utility for
  liveness), `notifications/resources/list_changed`, and its two transports (stdio; Streamable
  HTTP, whose server messages arrive on a GET stream). Rusty (MIT), read at `daf15f5`:
  `rusty-mcp/src/main.rs` (`json_result` :36-41 gives each answer as JSON text in a text block;
  `SettingEntry` :523-530; `looks_secret` :542-547 masks credential-like keys; `mutate` :734-742
  announces a change after each write; `settings_list` :988-1004; `setting_set` :1850-1861;
  `get_info` :1893-1911 declares `resources.listChanged`; `spawn_change_notifier` :2060-2082;
  `main` :2087-2124: stdio with no arguments, `--http [ADDR]`, `127.0.0.1:4174` by default);
  `rusty-core/src/brain/semantic.rs` :1-9 and :180-216 (the provider's values and how Rusty reads
  them); `rusty-core/src/core.rs:153-166` (the embedder is cached for 60 s);
  `rusty-app/qml/SettingsPage.qml:48-79` (Rusty's own page: its known keys with their wording and
  fallbacks, `settings_list` read again after a write and on each change).
- **Code we already ship:** `context_server` (`ContextServer::stdio` and `http`, `Ping`,
  `on_notification`, `InitializeResponse::server_info`; `StdioTransport`'s `Drop` kills the child,
  `stdio_transport.rs:144-148`; `transport/http.rs` sends POST and DELETE only, `:108`, `:376`).
  `marley_workbench::harness` (#534, #632: the follow loop, the 1 to 60 s wait, `Connection` and
  `Runtime` with their reasons, `find_rh` with `MARLEY_RH`). `fleet_providers.rs:360-418` (a stdio
  and an HTTP MCP client side by side) and `mcp_value`. `rusty.rs` (#633: `RustyOffer`, `offer`,
  `settle`). `settings_ui`: `SubPageLink` (`settings_ui.rs:1699-1712`), the window observing a
  global of another crate (`SkillIndex`, `:1868`), `render_mcp_servers_page` drawing
  `ui::AiSettingItem`, and `marley_page.rs`'s `ActionLink` naming a Marley action by string, the
  way the UI crate reaches Marley without depending on it. `ui::DropdownMenu`. `migrator`:
  `migrate_settings`'s list, `migrations::migrate_settings` (the root, the release and platform
  overrides, the profiles) and `m_2026_08_30::nest_markdown_preview_settings`, which moves keys
  into a nested block with `entry().or_insert`; `zed/src/zed/migrate.rs` (the banner, with a
  backup). `marley_sdk`'s `fixtures/` (`include_str!`'d contract answers). Python helpers Marley
  already ships (`marley_workbench/bin/marley-open-url`, the plugin's `event.py`). `which`
  (Cargo.lock). Ely GPUI Components at `2f8b2f6`: `src/feedback/status.rs` (`ConnectionStatus`, a
  dot and a word) and `src/settings/section.rs` (`SettingsRow`: title, description, control), read
  and not ported, since Zed's `ui::AiSettingItem` and the settings window's own rows cover both and
  Zed's own wins (R-D10). The sweep's answer: `context_server` owns the client and both transports,
  `settings_ui` owns the sub-page, `migrator` owns the carry-over; nothing new is needed under
  Marley's own code.

## UI proof
`script/e2e/643-rusty-switch-and-connection.sh` (`compositor sway`: it clicks a dropdown in the
Settings window). Fixtures: a scratch repository opened with `open_path`; `marley_rusty`'s stand-in
linked as `$E2E_WORK/bin/rusty-mcp` and named by `MARLEY_RUSTY_MCP` (L-531: Marley's search path
comes from the login shell, where the user's own `rusty-mcp` can come first), its state in
`$E2E_WORK/rusty`: Rusty's settings seeded from `fixtures/settings_list.json`
(`embedding_provider: "ollama"`) and a log of every request with the instance's pid; a keymap in
the run's profile binding Ctrl+Alt+Shift+R to `zed::OpenSettingsAt` with the sub-page's path and
Ctrl+Alt+Shift+M to `context_servers` (L-633). Setup checks the harness's copy (Rusty off, no
`rusty_tools`, `MARLEY_RUSTY_MCP` naming no file), then deletes `marley.rusty` from it so the run
starts on the shipped defaults. Every change to Marley's settings is an edit of the run's file
from outside; the dropdown writes to the stand-in, never to `settings.json` (L-607). Never the
user's Rusty (R-D8). Shots:
- `643-01-off`: the Rusty's Server sub-page with nothing set: Rusty off; the stand-in's log empty.
- `643-02-carried`: `marley.rusty_tools: true`, the old key, written; the Marley page searched for
  "Rusty": the Rusty section with Rusty and Rusty Tools for Agents on, Connection Embedded, and
  the sub-page's link.
- `643-03-connected`: the sub-page: connected, the stand-in's name and version, embedded, its path;
  Embedding Provider Ollama, read from the stand-in; the log holds `settings_list`.
- `643-04-written`: Off picked in the dropdown: the row reads Off; the log holds `setting_set` with
  `embedding_provider` and `off`, then `settings_list`.
- `643-05-refreshed`: the stand-in's stored provider changed to `openai` from outside, which makes
  it send `list_changed`: the row reads OpenAI with no step in Marley.
- `643-06-agent-tools`: the MCP Servers page: `rusty` running beside `marley`.
- `643-07-restarted`: Marley's stand-in instance killed (the pid that called `settings_list`):
  after the wait, connected again; the log shows a new pid's `initialize` and `settings_list`.
- `643-08-missing`: the stand-in moved away and Marley's instance killed: the sub-page says
  `MARLEY_RUSTY_MCP` names a file that is not there.
- `643-09-off-live`: `marley.rusty.enabled` false: Rusty off; no stand-in instance alive.
- `643-10-no-agent-tools`: the MCP Servers page: `marley` alone, with `agent_tools` still true.
- `643-11-on-again`: the stand-in back, `agent_tools` false, `enabled` true: connected; the
  provider still OpenAI.
- `643-12-service`: a stand-in the scenario starts with `--http 127.0.0.1:0`, `service_url` set to
  it and `connection` to `service`: connected, the URL shown, the provider read over HTTP; no stdio
  instance alive.
Teardown ends the HTTP stand-in.

## Locked-In Decisions
- D1 — One block, `marley.rusty { enabled, connection, service_url, agent_tools }`. `enabled` is
  the master switch, off by default and read "off unless on", and `agent_tools` acts only while it
  is on, so off means no `rusty-mcp` at all, Marley's own or the one Zed would start for its agents
  (R-D0: "Off means nothing starts"). `service_url` joins R-D0's three keys: Rusty's own
  `--http [ADDR]` lets the address vary, and a scenario must point `service` at a stand-in, never
  at the user's running service. Rejected: `agent_tools` independent of `enabled` (off would still
  start a `rusty-mcp` for Zed's agents); a `MARLEY_RUSTY_URL` variable in place of the key (hidden
  from the Settings window, shaped for tests).
- D2 — The carry-over is a JSON migration in Zed's `migrator`, in a module of Marley's own
  (`migrations/marley.rs`, `move_rusty_tools_into_rusty`) added to `migrate_settings`'s list.
  Through `migrations::migrate_settings` (the root, the release and platform overrides, the
  profiles): `marley.rusty_tools: true` becomes `marley.rusty.enabled: true` and
  `agent_tools: true`; `false` becomes `agent_tools: false`; a key the user already set under
  `marley.rusty` wins; the old key goes. Zed runs it in memory at every load
  (`SettingsStore::parse_and_migrate_zed_settings`), so Marley and the Settings window read the new
  keys at once, and Zed's banner over `settings.json` offers to write the file with a backup.
  `true` turns `enabled` on so what the user chose keeps working under D1's master switch; the
  cost, Marley's own connection starting too, goes in the CHANGELOG. Rejected: reading
  `rusty_tools` as a fallback (the merged settings cannot tell `default.json`'s `agent_tools:
  false` from a user's, so the page would show off while the tools are offered); dropping the key
  with a note (a silent loss for whoever turned #633 on, the case #642 left R1 to carry); a
  date-named module beside Zed's (an upstream migration of the same date would collide).
- D3 — `rusty-mcp` is `MARLEY_RUSTY_MCP`, else the first on Marley's search path, looked up at each
  connect off the main thread; a miss says where Marley looked. #633's offer uses the same lookup.
  Marley's search path comes from the user's login shell (L-531), so a scenario names its stand-in
  in the variable, as `MARLEY_RH` (#632), `MARLEY_GH` and `MARLEY_CLAUDE` do.
- D4 — The client is Zed's: `ContextServer::stdio` (id `marley-rusty`, `rusty-mcp` with no
  arguments) for `embedded`, `ContextServer::http` for `service`, its start bounded at 5 s. Liveness
  is an MCP `ping` every 5 s, bounded at 5 s, since Zed's client sees no server exit (L-534). A
  failed start, a failed ping, an exit or a missing program shows down with its reason and is tried
  again after 1, 2, 4 … 60 s, the wait reset by a good connect (#534's). Turning off or changing
  the source drops the server, whose stdio transport kills the child (`StdioTransport`'s `Drop`).
  The spawn is inside `context_server`, so gate:22's spawn sites do not change.
- D5 — Live refresh: the embedded connection subscribes to `notifications/resources/list_changed`
  (`on_notification`, as Zed's agent does for `tools/list_changed`) and reads Rusty's settings
  again. The service connection gets no notification, since Zed's HTTP transport opens no GET
  stream, so it reads on connect and after each write. Nothing depends on `changes_since`.
- D6 — Rusty's settings stay Rusty's: read with `settings_list`, written with `setting_set` and read
  again, never copied into `settings.json` (D11: Rusty is the only writer of its data). This ticket
  shows `embedding_provider` only, as a dropdown of Rusty's values `auto`, `ollama`, `openai` and
  `off`, read as Rusty reads them (`semantic.rs:185-216`: trimmed and lower-cased; `off`, `none`
  and `false` are off; anything else is `auto`; no stored value is `auto`) and written by the
  canonical name. Marley adds no fallback (Chad's decision 3 in the plan).
- D7 — Where it shows: a Rusty section on the Marley page, after System One, with four settings
  items (Rusty, Connection, Service URL, Rusty Tools for Agents, moved from Agents) and a sub-page
  link, Rusty's Server. The sub-page is a view `marley_workbench::rusty` draws and registers in
  `settings_ui::MarleyPageViews`, a global map from a sub-page's name to a view, because the UI
  crate cannot depend on a Marley crate. The state row is Zed's `ui::AiSettingItem`, as the MCP
  Servers page draws each server; the dropdown is `ui::DropdownMenu`. Rejected: a new
  `SettingsPageItem` variant drawing the state inside the section (several new match arms in
  `settings_ui.rs`); a Marley tab behind an `ActionLink`, as System One's Decisions is (the plan's
  R-D0 and R-D3 put Rusty's settings on the settings page); the settings UI drawing rows from data a
  Marley crate pushes (it would learn Rusty's meanings); porting Ely's `ConnectionStatus` and
  `SettingsRow` (Zed's own cover them).
- D8 — `crates/marley_rusty` (`[lib] path = "src/marley_rusty.rs"`, `MIT OR Apache-2.0`, no gpui,
  rustal's lint table) holds, in this ticket, the `settings_list` view and `EmbeddingProvider`, the
  names of the tools it reads, `fixtures/` (answers in `rusty-mcp`'s shape: JSON text in a text
  block), and the stand-in `stand_in/rusty-mcp`: a Python program on the standard library that
  answers from the fixtures over a scratch state folder, on stdio or with `--http ADDR` as
  `rusty-mcp` takes them, logs each request with its pid, and sends `list_changed` after a write and
  when its state file changes. Python because `just build` and the e2e runner build only `marley`,
  a Rust binary would need its own cargo build and an HTTP server crate, and Marley already ships
  Python helpers under its crates. Rejected: the real `rusty-mcp` on a scratch HOME (it ties the
  public repository's scenarios to an install, and Rusty's `Core::init` starts its watchers and
  indexer and reaches for Ollama).
- D9 — No scenario reaches the user's Rusty (R-D8): `script/e2e.sh` writes `marley.rusty: {
  enabled: false, service_url: "http://127.0.0.1:9/mcp" }` into each run's copy, deletes
  `marley.rusty_tools` from it, and exports `MARLEY_RUSTY_MCP` naming a file that does not exist; a
  scenario that turns Rusty on names its own stand-in and service. 633's and 642's scenarios move
  to the new keys and the shared stand-in.
- D10 — `service_url` must be an `http` URL on this machine (`127.0.0.1`, `localhost`, `[::1]`), as
  the push server's is (#535): `rusty-mcp` has no authentication and its tools read and write the
  brain. Any other URL shows down with that reason and nothing connects.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `marley.rusty.enabled` is not set, the system shall run no `rusty-mcp`, open no connection to Rusty, and show Rusty off on the Rusty's Server sub-page. | Shot `643-01-off`; the stand-in's empty log |
| REQ-002 | WHERE the user's settings hold `marley.rusty_tools: true` and no `marley.rusty`, the system shall read Rusty and Rusty Tools for Agents as on. | Shot `643-02-carried` |
| REQ-003 | WHERE the user's settings hold `marley.rusty_tools` beside a `marley.rusty` key it would set, the system shall keep the `marley.rusty` key, and a `false` shall carry as `agent_tools` off. | Review of the migration |
| REQ-004 | The Marley page shall list Rusty Tools for Agents in the Rusty section and not in the Agents section. | Shot `643-02-carried`; review of `marley_page.rs` |
| REQ-005 | WHILE Rusty is on with `connection: embedded`, the system shall start the `rusty-mcp` that `MARLEY_RUSTY_MCP` names, else the search path's first, on stdio, and the sub-page shall show it connected with the server's name, version and path. | Shot `643-03-connected` |
| REQ-006 | WHEN Marley connects, the sub-page shall show Rusty's `embedding_provider` as `settings_list` gives it, and `auto` where Rusty stores none. | Shot `643-03-connected`; review for the missing key |
| REQ-007 | WHEN the user picks an embedding provider on the sub-page, the system shall write it with `setting_set` and show the value read back. | Shot `643-04-written`; the stand-in's log |
| REQ-008 | WHEN the embedded server sends `notifications/resources/list_changed`, the sub-page shall show Rusty's settings read again, with no step by the user. | Shot `643-05-refreshed` |
| REQ-009 | WHILE Rusty and `agent_tools` are both on, the system shall offer `rusty` to Zed's agents, found as REQ-005 finds it. | Shot `643-06-agent-tools` |
| REQ-010 | IF the `rusty-mcp` Marley runs exits, THEN the sub-page shall say the connection is down and why, and the system shall start it again after 1, 2, 4 … 60 s. | Shot `643-07-restarted`; the log's new pid; review of the wait |
| REQ-011 | WHERE `rusty-mcp` cannot be found, the sub-page shall say so and where Marley looked. | Shot `643-08-missing` |
| REQ-012 | WHEN `marley.rusty.enabled` turns off while Marley runs, the system shall end its `rusty-mcp`, close the connection, withdraw the `rusty` offer whatever `agent_tools` says, and show Rusty off, without a restart. | Shots `643-09-off-live`, `643-10-no-agent-tools`; the process check |
| REQ-013 | WHEN `marley.rusty.enabled` turns on while Marley runs, the system shall connect without a restart. | Shot `643-11-on-again` |
| REQ-014 | WHILE `connection` is `service`, the system shall start no `rusty-mcp`, connect to `service_url`, and show it connected with the URL and Rusty's settings read over HTTP. | Shot `643-12-service`; the process check |
| REQ-015 | IF `service_url` is not an `http` URL on this machine, THEN the sub-page shall say so and the system shall connect to nothing. | Review |
| REQ-016 | IF a call to Rusty fails or Rusty refuses a setting, THEN the sub-page shall show the failure's first line. | Review |
| REQ-017 | The e2e harness shall write Rusty off and a service URL that reaches nothing into each run's copy of the user's settings, delete `marley.rusty_tools` from it, and name no `rusty-mcp` in `MARLEY_RUSTY_MCP`. | The scenario's setup checks; review of `script/e2e.sh` |
| REQ-018 | `crates/marley_rusty` shall build with `[lib] path = "src/marley_rusty.rs"`, `MIT OR Apache-2.0`, rustal's lint table and no gpui dependency. | Review; `script/gates.sh --diff` (gate:2, gate:9, gate:14, gate:17) |

## Phase Plan
- **P1 Plan** — promote once #642 completes (it turns `rusty_tools` off by default and edits
  `script/e2e.sh`); re-verify that `zed::OpenSettingsAt` opens a sub-page of the Marley page by its
  path, that a view made at app level redraws in the Settings window when it notifies, and whether
  `StdioTransport`'s shell execs the program so its `Drop` ends it; ask the brain (`brain_ask`) on
  D1, D2 and D7; confirm with Chad the fourth key (`service_url`) and that `enabled` gates
  `agent_tools`, with the migration turning `enabled` on for a carried `true`.
- **P2 Code** — the `README.md` marker first; the ledger rows widened and added before their files;
  the crate and its stand-in; the block, its default and the migration; `MarleySettings::rusty`;
  the client, the view and its registration, the offer's new rule; the section, the sub-page link
  and the dropdown's renderer; `zed.rs`'s test line; `script/e2e.sh`, 633's scenario and 642's; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG (Added; Changed: `rusty_tools` moved) and the user and architecture
  docs the notes list (§21), ledger capture (§19), the brain's #633 decision followed up as
  revised, close the ticket, archive, commit.
