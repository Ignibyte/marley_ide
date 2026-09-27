# The System One layer: typed decisions, off by default — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-565-system-one-layer.md
- **Pipeline spec:** 565-system-one-layer.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-26, approving the System One layer of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md` and its seven ranked uses, with three
  rules quoted in the spec's D1 to D4: local first, every aspect configurable and off by default
  so Marley never requires a model, and TypeSafe direct with data leaving only for listed
  projects (#516's redaction; a metadata-only mode for projects holding other people's data). His
  answer of the same day on what leaves the box: "Jev sees only the short updates the rail shows
  (the prompt and the last message cut to 300 characters with #516's redaction, the tool and what
  it acts on, states), never files or whole transcripts; a metadata-only mode sends states and
  tool names alone." This ticket is the note's use 0; the uses follow in #566 to #568.
- **Classification / tier:** feature, prong 2. A new Marley crate, two new workbench modules, a
  settings block, a settings page section. Four Zed paths, each with a row already:
  `Cargo.toml`, `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Size M to L; the seam for a split is the Decisions view.
- **Recall (§18.3):**
  - PR (prevention-rules.md:537, the forge client's bearer): a security invariant named in a
    decision is enforced in code, not documented: the destination of a secret is checked before
    the secret is used, `Debug` is manual so the key never prints, and no public getter hands the
    key out. D7 and the design's endpoint check follow it.
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001 and
    `marley_mcp`'s crate doc: a pure protocol core plus a masked transport, and no third-party
    SDK that brings tokio and axum into a gpui app. The layer keeps to `http_client` and reads
    `s1-rs` for its shapes only.
  - AD-claude-516-redact-at-the-tool-boundary-on-by-default-001: what leaves for a model is
    masked at the boundary, never in the buffer; the state builder masks every text value.
  - AD-claude-513-one-marley-per-data-directory-001 and L-claude-513: the data directory is per
    instance and `--user-data-dir` moves it, so the log lands in the run's profile in a scenario.
  - AD-claude-fleet-rail-quiet-no-transport-until-brain-001: a feature with no upstream feed stays
    quiet and honest; here, with the layer off, nothing is drawn.
  - L-claude-515-dispatch-through-the-window-from-inside-an-action-001: the two palette actions
    dispatch through the window, as `marley: open settings` learned to.
  - L-claude-516-fake-secrets-are-put-together-at-run-time-001 (the scenario's token) and
    L-claude-516-a-second-window-in-sway-narrows-the-terminal-under-test-001 (the Settings
    window opens after the terminal's part of the scenario).
  - lessons.md:2109: gpui runs a global's observers in the order they were registered; the
    settings observer that rebuilds the layer's configuration registers before the one that
    rebuilds the redactor, since the state builder takes the redactor as it is.
  - The memory note on `script/clippy`: cargo-shear fails on a workspace dependency with no
    user, so `marley_system_one`'s entry and its first use land in one change.
  - prevention-rules.md:692: `dirs` pulls an MPL crate; the crate takes its directory as an
    argument (`paths::data_dir()` from the workbench) and depends on no directory crate.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery** (each seam opened on 2026-09-26 at the working tree of `ca70b6488d` with #547 in
  Test; line numbers are of that tree):
  - `crates/http_client/src/http_client.rs`: `RequestTimeout` (31), `HttpRequestExt::timeout`
    (56, 65), `HttpClient::send` (128), `post_json` (154), `FakeHttpClient` behind
    `test-support` (424). `crates/gpui/src/app.rs`: `http_client` (1740), `write_credentials`
    (1585), `read_credentials` (1595), `delete_credentials` (1600).
  - `crates/anthropic/src/anthropic.rs:496-520`: the POST idiom the adapter copies in shape
    (builder, headers, `AsyncBody::from`, `client.send`, the host kept for the error).
  - `crates/language_model/src/api_key.rs`: `ApiKeyState` (19), `load_if_needed` (160: a
    non-empty environment variable wins, else the keychain by URL), `ApiKey::from_env` (233).
    `crates/env_var/src/env_var.rs:11` (`EnvVar::new`: empty is `None`).
    `crates/zed_credentials_provider/src/zed_credentials_provider.rs:22`, `:45-64`: on the dev
    channel the development provider (a file in the config directory) unless
    `ZED_DEVELOPMENT_USE_KEYCHAIN`; hence gpui's keychain directly. `Cargo.lock`: `gpui_linux`
    depends on `oo7` (line 7619), the secret-service client.
  - `crates/marley_mcp/src/redact.rs`: `Redactor::new` (135), `redact` (152), `Redacted` (117).
    `crates/marley_workbench/src/mcp.rs`: `McpServer` global and `start` (67), `refresh_redaction`
    (267, observing `SettingsStore`), `agent_redactor` (297), `for_agents` (306), `show_failure`
    with a toast (240, 247).
  - `crates/settings_content/src/marley.rs`: `MarleySettingsContent` (10), `MarleyLayout` with
    `strum::VariantArray` and `VariantNames` (33-46) for the page's dropdown.
    `crates/marley_workbench/src/marley_workbench.rs`: `actions!` (66), `MarleySettings` (168),
    `from_settings` (180-197), `init` (241), a toast (430).
    `crates/settings_ui/src/marley_page.rs`: `marley_page` (6), `agents_section` (42) with the
    `pick` and `write` closures; `crates/settings_ui/src/settings_ui.rs`: `SettingsPageItem`
    (1087), `ActionLink { title, description, button_text, on_click, files }` (1701),
    `SubPageLink` (1679). `assets/settings/default.json:1664-1674`: the `marley` block.
  - `crates/paths/src/paths.rs`: `set_custom_data_dir` (104), `data_dir` (145);
    `crates/zed/src/main.rs:263` (`--user-data-dir`); `script/e2e.sh:262-278` (`launch_marley`
    passes `--user-data-dir "$E2E_PROFILE"`, so `<data>` is the profile in a run).
  - `crates/marley_workbench/src/browser.rs`: `paths::data_dir().join("browser")` (627, 2642);
    `impl Item for BrowserView` (4788: `tab_content_text`, `tab_icon`, `deactivated`,
    `on_removed`); `register_serializable_item` (5386); `add_item_to_active_pane` (5748);
    `LastTerminal` (5400-5420, a global of the view and its window, set on focus).
  - `crates/editor/src/editor.rs:9012` (`set_masked`, for the key editor).
  - `crates/marley_workbench/src/browser_tools.rs:137` (`cx.background_executor().timer`).
  - Root `Cargo.toml`: members 140-148; `[workspace.dependencies]` Marley entries 421-428;
    `http_client` 387, `base64` 588, `chrono` 599, `futures` 649, `serde_json` 849, `sha2` 858.
    `crates/marley_workbench/Cargo.toml:11-52`: `chrono`, `paths`, `serde_json`, `settings`,
    `workspace` already; `http_client`, `env_var` and `marley_system_one` are new.
  - `docs/marley/zed-touchpoints.md`: the rows at 46 (`Cargo.toml`), 55 (`marley.rs`), 58
    (`marley_page.rs`) and 61 (`default.json`), each widened by this ticket.
  - TypeSafe's reference and models page, read on 2026-09-26 (the spec's prior art).
- **Decisions:** D1 to D10 in the spec.

### Design
- **The crate** (`crates/marley_system_one`, files `src/marley_system_one.rs`, `question.rs`,
  `state.rs`, `request.rs`, `reading.rs`, `policy.rs`, `log.rs`, `replay.rs`):
  - `UseSpec { name: &'static str, set: &'static QuestionSet, deadline: Duration }`; `Mode`
    (`Off`, `Shadow`, `Suggest`, `Act`) with serde in snake case and `strum` for the dropdown;
    `Provider` (`Typesafe`, `Compatible`, `Rules`, `Replay`); `Config` (the settings as
    resolved: provider, endpoint, model, projects, metadata-only projects, budget, price, modes).
  - `QuestionSet::instantiate(&self, fill: &Fill) -> Questions` fills a template's dynamic
    options; `request::build` renders `{model, state, questions}`; `request::parse` reads
    `answers` and `usage`, tolerant of fields it does not know.
  - `StateBuilder::new(project, detail).fact(label, value).text(label, value)` masks each text
    through `&dyn Fn(&str) -> String` and renders `label: value` lines; `hash()` is the SHA-256
    of the rendered state; `is_empty()` refuses a state with no fact and no text.
  - `reading::read(set, answers) -> Answered` applies each question's threshold and yields
    `Model { .. }` or `NoSignal { reason }`.
  - `policy::Gate { budget, breaker, bucket, dedupe }` with `fn admit(&mut self, use, subject,
    state_hash, now) -> Result<(), Refusal>` and `fn note(&mut self, outcome, now)`; pure over an
    injected clock in milliseconds.
  - `log::CallRow`, `OutcomeRow`, `append_in(dir, &Row) -> io::Result<()>`, `read_day_in(dir,
    date)`; `replay::Replay::load_in(dir)` and `answer(set, state) -> Option<Answers>`.
- **The adapter** (`crates/marley_workbench/src/system_one.rs`): `SystemOne` global with
  `config`, `key: Option<Key>` (a newtype with a manual `Debug` printing `Key(***)` and no
  getter beyond `header_value()` used at one site), `key_source`, `gate`, the log sender, and
  `recent: VecDeque<CallRow>` for the view. `init(cx)`: reads the settings, registers the
  `SettingsStore` observer (rebuilds `config`; reloads the key when the provider or endpoint
  changes), reads the environment variable, else spawns the keychain read; starts the log task
  (a background task owning the folder and appending each row it receives). `ask(use, state,
  verdict, cx) -> Task<Reading>`: policy first (`Refused`), dedupe, then by provider; the HTTP
  path builds the request with `HttpRequestExt::timeout(use.deadline)` and races it with a
  background timer; a 429 or 529 retries once if time remains; a 401 sets `key_refused` until
  the key changes; the body is parsed, read, logged and returned. The subject of a dedupe is the
  use's name and a subject key the use supplies (the seat, the tab, the entry).
- **The check** (`check/1`, deadline 2 s): the `SystemOneCheck` action reads `LastTerminal`, its
  workspace's project name and root, the terminal's title and its last block (`terminal.blocks()`,
  the command and `exit_code`), builds the state (`project`, `terminal`, `program` and
  `exit code` as facts; `last command` as text) with the use's own verdict (`command_failed` from
  the exit code), asks, and shows the reading in a toast.
- **The Decisions view** (`crates/marley_workbench/src/decisions.rs`): `DecisionsView`, a
  `workspace::Item` like `BrowserView` (tab text "Decisions", not serialized: it reopens from
  the palette), opened by `OpenDecisions` into the active pane; a header (calls and spend today
  against the cap, the key's source, provider and model, the breaker), the three buttons, the
  masked key editor, and a list of the day's rows from `read_day_in` plus the global's recent
  rows, each expandable. It observes the global for new rows.
- **Settings.** `SystemOneSettingsContent` (nested in `MarleySettingsContent` as `system_one`),
  the two enums with the `strum` derives `MarleyLayout` carries; `MarleySettings` gains
  `system_one: SystemOneSettings` resolved with the spec's defaults. The page's section uses
  `SettingItem`s whose `pick` and `write` walk `marley.system_one`; the map-keyed mode item
  reads `uses.get("check")` and writes `uses.insert("check", mode)`; the lists stay in
  `settings.json`, as #516's patterns do, and the description says so.
- **The scenario's fake server** (`$E2E_WORK/systemone.py`): reads the JSON body, logs the
  headers and body, answers `{model, answers: {command_failed: {type: "noul", noul: 0.9}},
  usage: {input_tokens: 1200, output_tokens: 0}}`, or sleeps or fails by the state's text; it
  prints its port for setup to write into the profile's settings. A `system_one_setting <key>
  <json>` helper (Python, like 516's `marley_setting`) rewrites one key of the block while
  Marley runs.
- **File manifest.** New Marley crate: `crates/marley_system_one/{Cargo.toml, src/*.rs}`.
  Marley crates: `crates/marley_workbench/src/system_one.rs` (new), `decisions.rs` (new),
  `marley_workbench.rs` (modules, the two actions, `MarleySettings`, `init`), `Cargo.toml`
  (`marley_system_one`, `http_client`, `env_var`). Zed paths: root `Cargo.toml`,
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Generated: `Cargo.lock`. Scripts:
  `script/e2e/565-system-one-layer.sh`. Docs at Complete:
  `docs/marley_architecture/marley_system_one.md` (new), `marley_workbench.md`.
- **Ledger rows** (`docs/marley/zed-touchpoints.md`, written before the Code step that needs
  them): the `Cargo.toml` row names ten members and `marley_system_one` among the workspace
  dependencies; the `marley.rs` row gains the `system_one` block and its enums; the
  `marley_page.rs` row gains the System One section; the `default.json` row gains the block's
  defaults.

### E2E plan
Fixtures: a scratch repository as the project; a HOME whose `.bashrc` is the scenario's; the
fake server started by setup with its port written into the profile's settings; the layer
enabled on `compatible` with the repository listed; the key exported. Each check is `marley:
system one check` from the palette, two seconds after a command in the terminal.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-010 | `marley: open settings`, the Marley page scrolled to System One | `565-01-settings-section` |
| REQ-002, REQ-009 | `echo hello`; the check; `marley: decisions`; the row clicked open | `565-02-decisions-view`; `holds` on the fake's log; `grep` for the key's value finds nothing |
| REQ-003, REQ-011 | `system_one_setting projects []`; `echo again`; the check | `565-03-refused-not-listed`; the fake's log unchanged |
| REQ-004 | the project listed again; `echo GITHUB_TOKEN=<a ghp_ token assembled at run time>`; the check | `565-04-masked`; the fake's log holds `[redacted: secret]` |
| REQ-005 | `system_one_setting metadata_only_projects [<repo>]`; `echo hidden`; the check | `565-05-metadata-only`; the body holds `exit code` and no `hidden` |
| REQ-006 | the project back on the full list; `echo slow`; the check; then `echo fail` and the check five times; a sixth `echo fail` and the check | `565-06-unavailable`; the rows' reasons |
| REQ-007 | `system_one_setting daily_budget_cents 0`; `echo hello`; the check | `565-07-budget`; the fake's log unchanged |
| REQ-008 | `system_one_setting provider "replay"` and a replay row written to `$E2E_PROFILE/system_one/replay.jsonl`; `echo hello`; the check | `565-08-replay` |
| REQ-001 | `system_one_setting enabled false`; `echo hello`; the check | `565-09-off`; no new request, no new row |
| REQ-012 | Test's gate and regression runs | the gate's exit code; `just regress` |

Not reachable by a scenario: a real TypeSafe answer (the fake speaks the published shape; the
live request at promotion proves the parse against the real one, off the record), the keyring
path (the run has no secret service the scenario may write to; REQ-009's keyring half is proven
by review of the read path and by the environment path), and a day boundary for the budget
(reviewed: the day is the local date of each row).

### Risks
- The answer shape is the reference as read on 2026-09-26, and the SDKs may add fields; the
  parser ignores unknown fields, and the promotion's live request settles it before code.
- Omarchy's secret service: gpui's Linux keychain needs one running; without it the keyring read
  fails and the source reads `none` with the reason, and the environment variable is the path.
- A nested settings block on the Marley page: `settings_ui` renders scalars and `strum` enums;
  the lists and the map stay in `settings.json`, and a map-keyed `SettingItem` is new ground
  (the `pick` and `write` closures decide; if the UI cannot render it, the check's mode moves
  to `settings.json` with the description saying so, and REQ-010 loses that item).
- The check's terminal is `LastTerminal`, set on focus: with the focus in the Decisions view the
  last terminal is still the right one; with no terminal ever focused the check refuses with the
  reason.
- A `compatible` endpoint over plain `http` is allowed on loopback only, so a misconfigured
  endpoint cannot send the key in the clear; the check refuses with the reason.
- The layer's cost with every use on is about 25 cents a day (the note); the cap of 50 cents is
  above a normal day and stops a runaway loop.
- Size: if the slice runs long, the Decisions view is the second slice (the toast and the log
  file prove the layer; the view reads what is already written), and the spec's REQ-002's view
  half, REQ-009's header and the "Set key" path move with it.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
