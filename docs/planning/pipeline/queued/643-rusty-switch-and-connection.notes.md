# Marley connects to Rusty when Rusty is turned on — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-643-rusty-switch-and-connection.md
- **Pipeline spec:** 643-rusty-switch-and-connection.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02, on the research: "maybe rusty becomes Marley. We would take our
  rusty custom QML app and build it inside of marley. The mcp then lives inside of the marley ide
  but enabled/disabled", then "Plan it now". On this batch, 2026-10-03: "lets make a plan to begin
  the work and spec out the tickets" and "lets make sure we use the gpui components we found here",
  confirmed as "Queue all five". The plan is `docs/marley/rusty-in-marley.md`; this is its slice R1
  (R-D0 the switch, R-D1 the crate, R-D2 live refresh, R-D8 scenarios off the real brain), first of
  #643 to #647, after #640 to #642.
- **Classification / tier:** feature, medium: larger than the plan's "S" for R1, because the
  brief puts the crate, both connections, the carry-over and the stand-in in it, and nothing else
  can start without them. One shippable slice; no split. Zed-side changes are additive: three
  files whose `zed-touchpoints.md` rows exist are widened, `zed.rs`'s test hunk changes one line,
  `Cargo.toml`'s member count moves, and `migrator` gains three rows (one new file, two lines).
  New crate: `marley_rusty`. New dependency edges: `marley_workbench` on `marley_rusty` and on
  `settings_ui`. No new external crate.
- **Recall (§18.3):**
  - AD-claude-633-rustys-server-is-offered-where-installed-as-a-default-001: the offer to Zed's
    agents, its default entry and the user's own entry winning. This ticket keeps the offer and its
    `settle`, moves its switch into the block and gates it by `enabled` (D1), and changes how it
    finds `rusty-mcp` (D3).
  - AD-claude-632-marley-runs-the-harness-as-a-child-and-says-its-state-001 and
    AD-claude-534-the-harness-is-followed-by-polling-in-a-section-outside-the-rails-model-001: a
    program Marley runs, its state said with the reason, started again after 1 to 60 s, dropped
    when the setting turns off. D4 copies it.
  - AD-claude-565-the-system-one-layer-is-a-pure-core-behind-an-adapter-off-by-default-001: a pure
    crate behind a `marley_workbench` adapter, off by default, off doing nothing; the shape of
    `marley_rusty` and `rusty.rs`, and why the content enums live once in `settings_content`.
  - L-claude-534-zeds-mcp-client-sees-no-server-exit-001: `wait_for_shutdown` fires on a failed
    send only; poll with a bounded call. Hence `ping` every 5 s, bounded at 5 s (D4).
  - L-claude-531-marley-takes-its-path-from-the-login-shell-so-stand-ins-are-named-001: a stand-in
    first on a scenario's PATH can lose to the user's real program. Rusty is installed on the dev
    box in `~/.local/bin`, and a lost race here would call `setting_set` on the user's real Rusty.
    Hence `MARLEY_RUSTY_MCP` (D3) and the harness's default naming no file (D9).
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001: the
    harness turns Rusty off in each copy; a scenario turns it back on with its own stand-in.
  - L-claude-633-the-mcp-servers-page-opens-by-a-keymap-in-the-runs-profile-001: the scenario's
    keys for the MCP Servers page and, by the same action, the Rusty sub-page.
  - L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: every settings change
    in the scenario is an outside edit; the dropdown writes to Rusty, not to `settings.json`.
  - L-claude-565-a-scenario-reaches-the-users-own-keyring-001: the headless sway shares the user's
    session; by the same token a scenario's Marley can reach the user's Rusty service on
    `127.0.0.1:4174`, which runs on the dev box. Hence the dead `service_url` in each copy (D9).
  - L-claude-534-a-harness-scenario-refuses-the-root-to-hold-a-down-state-001: a client that
    reconnects in a second leaves no time to shoot it down; here moving the stand-in away holds the
    connection down for `643-08-missing`.
  - F-claude-634-zeds-tests-started-the-context-servers-marley-offers-001: Zed's tests turn
    Marley's offers off; with `rusty_tools` gone, the tests' hunk turns `marley.rusty` off instead.
  - L-claude-456-acting-around-zeds-own-settings-observers-001: an observer registered in
    `marley_workbench::init` runs before every view's, so `rusty::init`'s observer settles the
    connection before the Settings window redraws.
  - Completed pipelines 632 and 633 (spec and notes); queued #642 (its Out and Risks: "with the
    default off, R1 only has to carry the users who turned it on"; its scenario sets
    `marley.rusty_tools`); queued #646 (drafted beside this one: it assumes #643's stand-in "first
    on the PATH" and a fixture vault from #643; see Risks).
  - Brain (`rusty-cli brain search`, read only):
    `decisions/marley-offers-rusty-mcp-to-zeds-agents-where-it-is-installed`, revised by D1 to D3;
    `decisions/rusty-files-marleys-rq1-rq5-as-ticket-035-to-039-and-m10-the-qt-app-frozen`
    (TICKET-035, `changes_since`, not built: D5 depends on nothing from it).
  - The dev box's own Marley settings set neither `rusty_tools` nor `rusty` (checked), so the
    carry-over changes nothing here; it is for whoever turned #633's offer on elsewhere.
- **Discovery** (line numbers as of 2026-10-03, before #642 lands):
  - `crates/marley_workbench/src/rusty.rs`: module doc `:1-10`; `RustyOffer` `:26-30`; `init`
    `:34-38` observes the settings store; `offer` `:41-54` looks for `rusty-mcp` with
    `which::which_in` on `agents::launcher(cx).search_path` while wanted; `settle` `:58-90` puts
    `context_servers.rusty` into Zed's defaults or takes it out.
  - `crates/marley_workbench/src/marley_workbench.rs`: `pub mod rusty` `:72`; `MarleySettings`
    `:341-409`, `rusty_tools` `:375-376`; `EmbeddedHarness` `:462-479`; `RustyTools` `:481-497`;
    `from_settings` `:544-649` (`rusty_tools` `:585`); `init` `:741`, `harness::init` `:769`,
    `rusty::init` `:770`.
  - `crates/marley_workbench/src/harness.rs`: `Connection` `:75-83`, `Runtime` `:84-94`, `Source`
    `:95-104`, `Harness` `:106-121`; `init` and `follow_setting` `:157-194` (dropping the tasks
    stops the program); `find_rh` `:367-383` (`MARLEY_RH`, else the PATH, with the reason);
    `follow` `:406-443` (the 1 to 60 s wait); `connected` `:444-506` (`ContextServer::stdio`, the
    start raced against a 5 s timer); `call` `:544-577` and `answer` `:578-599` (a tool's text read
    as JSON).
  - `crates/marley_workbench/src/fleet_providers.rs:360-418` (`connect`: `ContextServer::stdio` and
    `ContextServer::http` with `cx.http_client()`), `:461-504` (`call`, `mcp_value`).
  - `crates/marley_workbench/src/agents.rs:57-85` (`Launcher::search_path` is the process's
    `PATH`); `github.rs:13-15` (`MARLEY_GH`: "Marley takes its PATH from the user's login shell").
  - `crates/marley_workbench/Cargo.toml`: `context_server`, `which`, `settings`, `zed_actions`; no
    `settings_ui` yet.
  - `crates/settings_content/src/marley.rs`: `rusty_tools` `:88-92`; `system_one` and `fleet`
    `:165-172`; `MarleyPushSettingsContent` `:241-250` (the derives a small block carries);
    `SystemOneSettingsContent` `:252-293` (a block whose `enabled` is the master switch);
    `MarleyBlockDensity` `:428-450` (an enum with the dropdown's strum derives).
  - `assets/settings/default.json`: the `marley` block from `:1665`; `embedded_harness` `:1706`;
    `rusty_tools: true` and its comment `:1750-1753` (false after #642); `system_one` `:1796`.
  - `crates/settings_ui/src/marley_page.rs`: `marley_page()` `:10-22`; Rusty Tools for Agents in
    `agents_section()` `:344-363`; `system_one_section()` from `:643`, its `enabled` `:646-670`;
    the `ActionLink` dispatching `marley::OpenDecisions` by name `:1090-1113`; `privacy_section()`
    `:1116`.
  - `crates/settings_ui/src/settings_ui.rs`: `mod marley_page;` `:3`; Marley's dropdown renderers
    `:558-580`; `SettingsPageItem` `:1107-1113`; `SubPageLink` with its `render` function
    `:1699-1712`; the window's observers `:1838-1890` (`SettingsStore`, `FeatureFlagStore`, and
    `SkillIndex`, a global of `agent_skills`).
  - `crates/settings_ui/src/page_data.rs:8740-8751`: the MCP Servers `SubPageLink`
    (`json_path: "context_servers"`, `in_json: false`); `zed_actions/src/lib.rs:171`:
    `AGENT_SKILLS_SETTINGS_PATH = "agent.skills"`, a sub-page path that names no real key.
  - `crates/settings_ui/src/pages/mcp_servers_page.rs:23-66`: a sub-page that draws each server
    with `ui::AiSettingItem` and its status; `crates/ui/src/components/ai/ai_setting_item.rs:6-15`
    (`AiSettingItemStatus`: Stopped, Starting, Running, Error, …) and `:72-81` (label, detail,
    actions, details).
  - `crates/context_server/src/context_server.rs:49-166` (`stdio`, `http`, `client`, `start`,
    `stop`); `protocol.rs:81-84` (`initialize`, with `server_info`) and `:140-146`
    (`on_notification`); `types.rs:79` (`Ping`) and `:120`
    (`notifications/resources/list_changed`); `transport/stdio_transport.rs:28-73` (the program
    started through `ShellBuilder`'s system shell) and `:144-148` (`Drop` kills it);
    `transport/http.rs:108` and `:376` (POST and DELETE only, no GET stream);
    `crates/agent/src/tools/context_server_registry.rs:142-158` (Zed's agent subscribing to a
    `list_changed`).
  - `crates/migrator/src/migrator.rs:159-260` (`migrate_settings`'s list, its last entry `:258`);
    `migrations.rs:7-48` (`migrate_settings` over the root, the release and platform overrides and
    the profiles) and `:374-378` (a migration's module); `migrations/m_2026_08_30/settings.rs`
    (keys moved into a nested block); `crates/migrator/Cargo.toml` (already on `serde_json` and
    `settings_content`).
  - `crates/settings/src/settings_store.rs:796-827` (`parse_and_migrate_zed_settings`: the user's
    file migrated in memory at each load; a write through the Settings window does not migrate the
    text first, which is why D2 lets a key under `marley.rusty` win);
    `crates/zed/src/zed/migrate.rs:25`, `:118`, `:241`, `:270-276`, `:303` (the banner, "Backup
    and Update").
  - `crates/zed/src/zed.rs:6229-6236`: the tests' settings, `marley.rusty_tools = Some(false)`
    (#634).
  - `script/e2e.sh:19-30` (`setup`, `teardown`, `open_path`), `:627-642` (the copy of the user's
    settings, `rusty_tools: false`); `script/e2e/633-rusty-tools-for-zeds-agents.sh:11-62`
    (`set_setting`, the inline stand-in, the keymap); #642's queued scenario sets
    `marley.rusty_tools` (`642-06-rusty-on`).
  - `crates/marley_fleet/Cargo.toml` and its `LICENSE-MIT`, `LICENSE-APACHE` (a pure crate's
    manifest, licence and lint table); `crates/marley_sdk/src/pseudo.rs:20-22` (fixtures
    `include_str!`'d); `crates/marley_workbench/bin/marley-open-url` (a Python helper under a
    Marley crate).
  - `script/gates.sh:99`, `:194` (the gates find Marley crates by `crates/marley_*`), `:361`
    (`SPAWN_SITES_PIN=7`, unchanged); `.config/spawn-sites.yml:17-30` (no `ContextServer` form).
  - `Cargo.toml:143-148` and `:426-431` (members, `[workspace.dependencies]`); its ledger row says
    "Eleven".
  - Rusty, `/srv/stacks/rusty-v3` at `daf15f5`: the lines in the spec's Prior art. `settings_list`
    lists stored keys only, so a missing `embedding_provider` means Rusty's default, `auto`.
- **Decisions:** D1 to D10 in the spec. In short: one block whose `enabled` gates everything,
  `agent_tools` included, with `service_url` added; the old key carried by a Zed migration in a
  Marley module, `true` turning Rusty on; `MARLEY_RUSTY_MCP` before the search path, for the offer
  too; Zed's client with a bounded `ping` and #534's wait; `list_changed` on stdio, reads on
  connect and write on HTTP; Rusty's settings read and written only through its tools, the
  embedding provider first; a Rusty section and a sub-page drawn by a Marley view the UI crate
  hosts; the crate with a Python stand-in; the harness off, unpointed and unnamed; loopback only.

### Design
- **`crates/marley_rusty`** (Marley crate, new):
  - `Cargo.toml`: `[lib] path = "src/marley_rusty.rs"`, `license = "MIT OR Apache-2.0"`,
    dependencies `serde` and `serde_json`, rustal's lint table as `marley_fleet` carries it;
    `LICENSE-MIT` and `LICENSE-APACHE` beside it.
  - `src/marley_rusty.rs`: the crate's doc (the pure core of Rusty in Marley: typed views of
    `rusty-mcp`'s answers, no gpui and no IO; later slices add theirs), the dylint `cfg_attr` block
    every Marley root carries, the tool names this ticket reads (`settings_list`, `setting_set`),
    `pub mod settings;`.
  - `src/settings.rs`: `ServerSettings` (the `settings_list` answer: `from_answer(&str)`, an error
    naming what did not parse, and `get(key)`); `EmbeddingProvider { Auto, Ollama, OpenAi, Off }`
    with its key (`embedding_provider`), `from_stored(Option<&str>)` read as `semantic.rs` reads
    it, `as_setting()` (the canonical name written back), `ALL`, and each value's label and
    description in Rusty's own words (`SettingsPage.qml:51`, `semantic.rs:1-9`: `openai` needs
    `openai_api_key` in Rusty's secrets and sends page text off the machine).
  - `fixtures/settings_list.json`: an answer in `rusty-mcp`'s shape (`[{"key", "value"}]`, sorted by
    key) with `embedding_provider: "ollama"` and `pin_timeout_minutes: "5"`; neutral values,
    nothing of the user's.
  - `stand_in/rusty-mcp` (executable, Python standard library only): stdio with no arguments,
    `--http ADDR` (port 0 allowed; the bound address written to `$RUSTY_STAND_IN_STATE/http-addr`).
    Answers `initialize` (server `rusty-mcp`, version `stand-in`; capabilities `tools` and
    `resources.listChanged`), `ping`, `tools/list` (`settings_list`, `setting_get`, `setting_set`
    and the brain loop's four, so 633's MCP Servers page keeps its list) and `tools/call` for the
    three settings tools in `rusty-mcp`'s shape (JSON text in a text block; an unknown tool is a
    JSON-RPC error, as rmcp answers). Its state lives in `$RUSTY_STAND_IN_STATE`: `settings.json`,
    seeded from the fixture (found beside the script through its real path, so a symlink works) on
    first start, and `calls`, one line per request with the pid, the method, the tool and its
    arguments. It sends `notifications/resources/list_changed` on stdio after a `setting_set` and
    when `settings.json` changes on disk (checked twice a second), standing for a change
    `rusty-mcp` sees. Over HTTP it answers a POST with `application/json`, a notification with
    202, a DELETE with 200, and opens no GET stream, since Zed's client asks for none.
- **Zed, `settings_content`** (`crates/settings_content/src/marley.rs`): `pub rusty:
  Option<MarleyRustySettingsContent>` after `system_one`, documented "Rusty in Marley (#643):
  Marley's connection to Rusty's MCP server, `rusty-mcp`. Off until it is turned on."
  `MarleyRustySettingsContent { enabled, connection, service_url, agent_tools }` with
  `with_fallible_options` and `MarleyPushSettingsContent`'s derives, each field's doc with its
  default; `MarleyRustyConnection { Embedded, Service }`, `Embedded` the default, with
  `MarleyBlockDensity`'s derives. `rusty_tools` removed.
- **Zed, `default.json`**: `rusty_tools` and its comment removed; after `system_one`, `"rusty": {
  "enabled": false, "connection": "embedded", "service_url": "http://127.0.0.1:4174/mcp",
  "agent_tools": false }` with a comment: off until you turn it on; on, Marley starts `rusty-mcp`
  on stdio (`embedded`: `MARLEY_RUSTY_MCP`, else your PATH) or connects to Rusty's running
  service (`service`: `service_url`, on this machine), and the Marley settings page shows its
  state and Rusty's own settings; `agent_tools` also offers Rusty's tools to Zed's agents.
- **Zed, `migrator`**: `crates/migrator/src/migrations/marley.rs` (new): `pub(crate) fn
  move_rusty_tools_into_rusty(value: &mut Value) -> Result<()>` through
  `migrations::migrate_settings`; per object, under `marley`: no `rusty_tools`, nothing; a
  `rusty` that is neither absent nor an object, the old key left for the schema to flag (as
  `m_2026_08_30` leaves a non-object); else `rusty` made an object, `agent_tools` inserted with
  the old value unless present, `enabled: true` inserted unless present when the old value is
  `true`, and `rusty_tools` removed. `migrations.rs`: `pub(crate) mod marley;`. `migrator.rs`:
  `MigrationType::Json(migrations::marley::move_rusty_tools_into_rusty)` in `migrate_settings`'s
  list; its place does not matter, since it reads and writes only `marley` keys. Each hunk carries
  a `// Marley:` comment.
- **Zed, `settings_ui`**:
  - `marley_page.rs`: `agents_section()` loses Rusty Tools for Agents (its array one shorter).
    `rusty_section() -> [SettingsPageItem; 6]`, chained after `system_one_section()`: the header
    Rusty; Rusty (`marley.rusty.enabled`: "Connect Marley to Rusty, the local assistant store,
    through its MCP server, rusty-mcp. Off, Marley starts no rusty-mcp, opens no connection and
    offers Zed's agents no Rusty tools."); Connection (`marley.rusty.connection`: embedded starts
    rusty-mcp on stdio, from MARLEY_RUSTY_MCP or the PATH, and ends it when Rusty turns off or
    Marley quits; service connects to Rusty's running service at the Service URL and starts
    nothing); Service URL (`marley.rusty.service_url`: where Rusty's service listens, on this
    machine); Rusty Tools for Agents (`marley.rusty.agent_tools`, #633's description plus "while
    Rusty is on"); a `SubPageLink` titled Rusty's Server (`json_path: Some("marley.rusty.server")`,
    a path that names no real key, as `agent.skills` does; `in_json: false`; `files: USER`;
    description: whether Marley is connected to rusty-mcp, and Rusty's own settings, read from and
    written to it; `render: render_rusty_server_page`).
  - `MarleyPageViews`, in `marley_page.rs`: a `Global` map from a sub-page's name to the `AnyView`
    a Marley crate registers, with `set` and `get`; `render_rusty_server_page` draws the `rusty`
    view in the sub-page's scroll container (`track_scroll` on the handle it is given, as
    `render_mcp_servers_page` does), or a muted line when no view is registered.
  - `settings_ui.rs`: `pub use marley_page::MarleyPageViews;` and
    `.add_basic_renderer::<settings::MarleyRustyConnection>(render_dropdown)` beside Marley's
    other dropdowns.
- **Zed, `zed`** (`crates/zed/src/zed.rs:6236`): `marley.rusty_tools = Some(false)` becomes
  `marley.rusty.get_or_insert_default().enabled = Some(false)`, its comment kept (#634's reason).
- **Marley, `marley_workbench`**:
  - `Cargo.toml`: `marley_rusty` and `settings_ui`.
  - `marley_workbench.rs`: `MarleySettings::rusty: rusty::RustySettings` in place of `rusty_tools`;
    `RustyTools` folds into it; `from_settings` reads `marley.rusty`.
  - `rusty.rs`, from #633's offer to Rusty's adapter (the module doc rewritten):
    - `RustySettings`: on or off; the source (`Off`, `Embedded`, `Service(url)`); whether the
      agents get the tools (only when on and `agent_tools`).
    - `find`: `MARLEY_RUSTY_MCP`, else `which_in` on the launcher's search path, in
      `background_spawn`; its error says where it looked, in #632's wording ("no rusty-mcp on the
      PATH, and MARLEY_RUSTY_MCP is unset"; "MARLEY_RUSTY_MCP names <path>, which is not there").
    - The `Rusty` global: the source, the state (`Off`, `Starting`, `Connected { server, via }`,
      `Down(reason)`, `Missing(reason)`), the last settings read (`ServerSettings` or the reason it
      failed), a refused write's line, the server, the keeper task and the `list_changed`
      subscription.
    - `init`: the global, `follow_setting` once and on every settings change, the view made and
      registered in `MarleyPageViews` under `rusty`. `follow_setting`: on a changed source, drops
      the task and the server (ending the child) and starts `keep`; then the offer as #633 runs it,
      wanted only when on with `agent_tools`, found by `find`.
    - `keep`: loop: connect (find, or check the URL is loopback `http` (D10); start within 5 s),
      `Connected` with `server_info`'s name and version and the path or URL; read the settings;
      subscribe to `list_changed` on stdio; `ping` every 5 s within 5 s until one fails; then
      `Down` or `Missing` with the root cause's first line, wait 1, 2, 4 … 60 s, again.
    - `read_settings` (`settings_list` within 5 s into `ServerSettings`) and `set_server_setting`
      (`setting_set` within 5 s, a refusal kept for the view, then `read_settings`), both through a
      `call` shaped like `harness::call` and `fleet_providers::mcp_value`.
    - `RustyServerView` (`Render`, observing the `Rusty` global): `ui::AiSettingItem` for
      `rusty-mcp` (Stopped while off, Starting, Running with "name version, embedded: path" or
      "service: url", Error with the reason); then "Rusty's settings": Embedding Provider with
      Rusty's description and a `ui::DropdownMenu` of the four values while connected and read, a
      muted line saying why not otherwise, and a refused write's line.
- **Marley, scripts:** `script/e2e.sh`: in the copy, delete `marley.rusty_tools` and set
  `marley.rusty` to `{"enabled": false, "service_url": "http://127.0.0.1:9/mcp"}`; export
  `MARLEY_RUSTY_MCP="$E2E_WORK/no-rusty-mcp"` before `setup`; the comment says why (L-633, L-531,
  R-D8). `script/e2e/633-rusty-tools-for-zeds-agents.sh`: the stand-in linked from
  `marley_rusty`, `MARLEY_RUSTY_MCP`, `marley.rusty.enabled` and `agent_tools` set in place of
  `rusty_tools`, `633-02-off` turning `agent_tools` off. 642's scenario likewise for its
  `rusty_tools` checks and steps, once it has landed.
  `script/e2e/643-rusty-switch-and-connection.sh` (new, Test phase).
- **File manifest.**
  - Marley: `crates/marley_rusty/{Cargo.toml, LICENSE-MIT, LICENSE-APACHE, src/marley_rusty.rs,
    src/settings.rs, fixtures/settings_list.json, stand_in/rusty-mcp}` (new);
    `crates/marley_workbench/Cargo.toml`, `src/marley_workbench.rs`, `src/rusty.rs`;
    `script/e2e.sh`; `script/e2e/633-rusty-tools-for-zeds-agents.sh`; 642's scenario;
    `script/e2e/643-rusty-switch-and-connection.sh` (new).
  - Zed: `crates/settings_content/src/marley.rs` (crate `settings_content`);
    `assets/settings/default.json`; `crates/settings_ui/src/marley_page.rs` and `settings_ui.rs`
    (crate `settings_ui`); `crates/migrator/src/migrations/marley.rs` (new), `migrations.rs`,
    `migrator.rs` (crate `migrator`); `crates/zed/src/zed.rs` (crate `zed`); `Cargo.toml` and
    `Cargo.lock` (the workspace).
- **The ledger rows** (`docs/marley/zed-touchpoints.md`, written before the code, §14):
  - `crates/settings_content/src/marley.rs` (`:59`): the "`rusty_tools: Option<bool>` … (#633)"
    fragment says it moved into `rusty.agent_tools` (#643); append "`rusty:
    Option<MarleyRustySettingsContent>` with `enabled`, `connection` (`MarleyRustyConnection {
    Embedded, Service }`, `Embedded` the default and the strum derives `MarleyLayout` has),
    `service_url` and `agent_tools`: Rusty's switch and connection (#643)".
  - `assets/settings/default.json` (`:67`): "`marley.rusty_tools: true` with its comment (#633)"
    gains "`false` since #642, moved into `marley.rusty` by #643"; append "`marley.rusty: {
    enabled: false, connection: "embedded", service_url: "http://127.0.0.1:4174/mcp", agent_tools:
    false }` with its comment (#643)".
  - `crates/settings_ui/src/marley_page.rs` (`:63`): the Agents section's Rusty Tools for Agents
    "moved to the Rusty section as `marley.rusty.agent_tools` (#643)"; append "a Rusty section
    after System One (Rusty, Connection, Service URL, Rusty Tools for Agents) and a Rusty's Server
    sub-page link whose page draws the view a Marley crate registers in `MarleyPageViews`, a
    global this file defines (#643)".
  - `crates/settings_ui/src/settings_ui.rs` (`:66`): append "the dropdown renderer for
    `settings::MarleyRustyConnection` and `pub use marley_page::MarleyPageViews` (#643)".
  - `crates/zed/src/zed.rs` (`:68`): the tests' Rusty line: "`marley.rusty.enabled = Some(false)`
    where `marley.rusty_tools = Some(false)` was (#643)".
  - `Cargo.toml` (`:47`): "Eleven" becomes "Twelve", and `marley_rusty` (#643) joins the
    `[workspace.dependencies]` list. `Cargo.lock` (`:48`): its row already covers the Marley
    crates' entries.
  - New rows: `crates/migrator/src/migrations/marley.rs` (new file: Marley's settings migrations,
    `move_rusty_tools_into_rusty` (#643) | a user's `marley.rusty_tools` carried into
    `marley.rusty` at each load, so Marley, the Settings window and the file agree, and Zed's
    banner offers to write it | keep the file; a name of Marley's own, so no upstream migration's
    date collides); `crates/migrator/src/migrations.rs` (`pub(crate) mod marley;` | the module |
    re-add the line); `crates/migrator/src/migrator.rs` (the Marley migration in
    `migrate_settings`'s list | so it runs at each load | keep it in the list; its place does not
    matter).

### Visual check plan
The scenario `script/e2e/643-rusty-switch-and-connection.sh`, `compositor sway`. Setup: a scratch
repository opened with `open_path`; `ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp"
"$E2E_WORK/bin/rusty-mcp"`; `export MARLEY_RUSTY_MCP=$E2E_WORK/bin/rusty-mcp
RUSTY_STAND_IN_STATE=$E2E_WORK/rusty`; the keymap (Ctrl+Alt+Shift+R to `zed::OpenSettingsAt {
path: "marley.rusty.server" }`, Ctrl+Alt+Shift+M to `context_servers`); `expect` the copy holds
`marley.rusty.enabled` false and no `marley.rusty_tools`, and that the harness's
`MARLEY_RUSTY_MCP` named no file (checked before the scenario's own export); then `del_setting
marley.rusty`. Checks read the stand-in's `calls` (`holds`) and test each logged pid with `kill
-0` from a shell function, never a `pgrep -f` wrapped in `bash -c` (the trap #642 names).

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | Trust the project; Ctrl+Alt+Shift+R | `643-01-off`: Rusty off; `calls` absent |
| REQ-002, REQ-004 | Close the window; `set_setting marley.rusty_tools true`; settle; `marley: open settings`; type "Rusty" | `643-02-carried`: the Rusty section, Rusty and Rusty Tools for Agents on, Connection Embedded, the link |
| REQ-005, REQ-006 | Ctrl+Alt+Shift+R | `643-03-connected`: Running, `rusty-mcp stand-in`, embedded, the path; Embedding Provider Ollama; `calls` holds `settings_list` |
| REQ-007 | Click the dropdown, click Off | `643-04-written`: Off; `calls` holds `setting_set` with `embedding_provider` and `off`, then `settings_list` |
| REQ-008 | Write `openai` into the stand-in's `settings.json` from outside; settle 3 | `643-05-refreshed`: OpenAI |
| REQ-009 | Ctrl+Alt+Shift+M | `643-06-agent-tools`: `rusty` with a green dot beside `marley` |
| REQ-010 | Kill the pid that logged `settings_list`; settle 15; Ctrl+Alt+Shift+R | `643-07-restarted`: Running; `calls` holds a new pid's `initialize` and `settings_list` |
| REQ-011 | Move the stand-in's link away; kill Marley's instance; settle 10 | `643-08-missing`: the reason names `MARLEY_RUSTY_MCP` and its path |
| REQ-012 | `set_setting marley.rusty.enabled false`; settle 4 | `643-09-off-live`: Rusty off; no logged pid alive; then Ctrl+Alt+Shift+M: `643-10-no-agent-tools`, `marley` alone |
| REQ-013 | Link back; `set_setting marley.rusty.agent_tools false`; `set_setting marley.rusty.enabled true`; settle 6; Ctrl+Alt+Shift+R | `643-11-on-again`: Running; OpenAI |
| REQ-014 | Start `rusty-mcp --http 127.0.0.1:0` in the background (its pid kept for teardown); read `http-addr`; `set_setting marley.rusty.service_url` to it and `marley.rusty.connection "service"`; settle 6 | `643-12-service`: Running, `service: http://127.0.0.1:<port>/mcp`, OpenAI; no stdio pid alive |
| REQ-017 | The setup's `expect` lines | their `pass` lines in the run's output |

Not reached by the scenario, and why: a real `rusty-mcp` (R-D8: the brain is private and the
repository public; the stand-in answers in its shape, from `main.rs`); Zed's migration banner
over `settings.json` (Zed's own, unchanged; the in-memory carry-over is what `643-02-carried`
shows); REQ-003, REQ-015, REQ-016 and REQ-018 are checked in the review and by the gate.

### Risks
- **A view drawn in the Settings window.** The sub-page draws an `AnyView` made at app level, and
  the Settings window must redraw when that view notifies. gpui invalidates every window that drew
  an entity when it notifies; promotion confirms it on this tree, and if it does not hold, the
  observer that changes the `Rusty` global also calls `cx.refresh_windows()`, as
  `block_headers::init` does.
- **`OpenSettingsAt` to a sub-page of the Marley page.** L-633 opened MCP Servers, a sub-page of
  Zed's own page, by its path. Promotion confirms the Marley page's sub-page opens the same way;
  else the scenario searches "Rusty's Server" and presses Enter.
- **The child behind a shell.** `StdioTransport` starts the program through `ShellBuilder`'s
  system shell, and its `Drop` kills the direct child. If that shell does not exec, the stand-in
  could outlive the switch; the scenario's pid check at `643-09` catches it, and the Code phase
  then ends the program's process group.
- **Two `rusty-mcp`.** With `agent_tools` on and `embedded`, Zed's context server and Marley's
  connection each run one. Rusty serves several processes on one store today (each agent's stdio
  instance); accepted, and named in the guide.
- **Turning Rusty on for a carried `true`** starts Marley's own connection for that user, which
  they did not ask for by that name (D2). The CHANGELOG's Changed entry says so; the dev box's own
  settings hold neither key.
- **Rusty caches its embedder for 60 s** (`core.rs:153-166`): a provider written takes effect in
  Rusty within a minute; the page shows the stored value at once.
- **The later drafts.** #644 to #647, drafted beside this one, put #643's stand-in "first on the
  PATH", and #646 expects a fixture vault from #643. D3 and D9 make `MARLEY_RUSTY_MCP` the way
  (the harness's default names no file, so a stand-in only on the PATH is never found), and the
  vault comes with #644, as #644's own draft already says. Each promotion takes this ticket's
  names: the variable, `RUSTY_STAND_IN_STATE`, the `calls` log and the sub-page's path.
- **#642 not landed.** This plan is written against #642's tree: `rusty_tools` false by default,
  `script/e2e.sh` writing Voice off, a Voice section before System One, and 642's scenario setting
  `rusty_tools`. Promotion re-reads those lines.
- **Timing in the scenario.** A kill is seen within one ping and its timeout (10 s at most) and
  retried after 1 s; `643-07` settles 15 s. `643-08`'s retries keep growing while the link is
  away, and `643-11` turns the switch off and on, which starts a fresh wait.
- **The receipt.** The stand-in, the fixtures and the scenarios are fingerprinted by the commit
  receipt (`crates/marley_*`, `script/e2e*`); a change to any after the Code phase's green needs
  `--diff` again before the commit.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: `:911-916` (Rusty's tools: now `marley.rusty.agent_tools`, in the Rusty
  section, only while Rusty is on; an old `rusty_tools` is carried); a Rusty part (the switch, the
  two connections, `MARLEY_RUSTY_MCP`, the sub-page, the embedding provider as Rusty's own
  setting); `:1712-1720` (the Settings page's sections) and the keys block after it.
- `docs/marley/walkthrough.md`: `:302` (the page's sections, with Rusty); a stop for Rusty where
  `rusty-mcp` is installed, if Complete finds a fit.
- `crates/marley_workbench/guide/index.html`: checked, no Rusty article; nothing to change.
- Architecture (§21): `docs/marley_architecture/marley_rusty.md` (new, as `marley_sdk.md` and
  `marley_system_one.md` are); `docs/marley_architecture/marley_workbench.md:1821-1831` (Rusty's
  tools become Rusty: the connection, the sub-page view, the offer);
  `docs/marley/three-prong-plan.md` D8 (its `marley_rusty` and `marley_brain` are one crate,
  R-D1) and the C2 row;
  `docs/marley/rusty-in-marley.md`'s slices table (R1, #643); the zed-touchpoints rows checked
  against what shipped; `CHANGELOG.md` (Added: Rusty in Marley's switch and connection; Changed:
  `marley.rusty_tools` is `marley.rusty.agent_tools`).

### Checklist (no TaskCreate in this harness)
- [x] Read the brief (both parts), CONSTITUTION §3, §7, §14, §18, §19, §20, and the ticket and
      pipeline templates.
- [x] Read `docs/marley/rusty-in-marley.md` in full (R-D0 to R-D10, the slices, Rusty's triage),
      and the plan's D8, D11, D19 and C2.
- [x] Read #633's spec, notes and `rusty.rs`; #632's spec and `harness.rs`; #642's queued spec and
      notes; #646's queued spec for what it assumes of #643.
- [x] Recall: the knowledge ledgers (ADs for #534, #565, #632, #633; L-456, L-531, L-534 twice,
      L-565, L-607, L-633 twice; F-634), the completed pipelines, a read-only brain search.
- [x] Discovery with file:line: the settings block, `default.json`, the Marley page, the settings
      UI's sub-pages and observers, `context_server`'s client and transports, `migrator` and the
      store's in-memory migration, `zed.rs`'s tests, the e2e harness and 633's scenario, the
      gates' crate discovery and spawn sites, Rusty's `main.rs`, `semantic.rs`, `core.rs` and Qt
      settings page.
- [x] Prior-art sweep, three legs: Warp's MCP map, Zed's and Orca's maps (none for these crates);
      the MCP specification and Rusty's source; Zed's `context_server`, `settings_ui`, `ui`,
      `migrator`, Marley's harness, fleet and offer code; Ely's `ConnectionStatus` and
      `SettingsRow`, not ported.
- [x] The carry-over decided and locked (D2), with what was rejected and why.
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D10, eighteen EARS rows, phase plan.
- [x] Design: approach, file manifest by crate, the ledger rows to widen and add, the visual check
      plan, risks, the docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.
