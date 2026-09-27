# The System One layer: typed decisions, off by default — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-565-system-one-layer.md
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
  settings block, a settings page section. Five Zed paths since promotion, each with a row
  already: `Cargo.toml`, `crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs`, `crates/settings_ui/src/settings_ui.rs` (the
  dropdown renderers) and `assets/settings/default.json`. Size M to L; the seam for a split is
  the Decisions view.
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

### Promotion (2026-09-27)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #521's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore) ✓; the answer shape checked ✓; spec and design updated ✓.
- **Recall, added:** PR-claude-a-zed-fork-identity-is-more-than-app-name-001 and
  AD-claude-437-marley-identity-is-app-name-and-the-dev-channel-001 (Marley stays on the `dev`
  channel, which keys the Secret Service items, so the keychain path is re-read below);
  PR-claude-redact-the-whole-text-before-cutting-it-001 (the state's text is masked whole, then
  cut to 300 characters); PR-claude-run-a-views-poll-while-it-shows-001 (#521: the Decisions view
  follows the global and polls nothing); L-claude-521-gpuis-mutable-global-access-tells-every-observer-001
  (the `SystemOne` global is written only when a row or the configuration changes). The brain
  (consultation de2f88036a50474db9ab866c2a613c28): no decision on the layer; its research pages
  name two Jev projects of Chad's, which gave the answer shape below.
- **The live request, replaced.** The plan's one live request with Chad's key cannot run: no key
  is in the environment, and key files are not read. Its purpose, the parse against the real
  answer, is met by what Chad's own projects recorded and send, read for their structure only
  (no values, no transcript text, no key file):
  - One project's recorded runs: 922 rows with answers, 574 of them from
    `jev-1.13.0`. A noul answers `{type: "noul", noul}` alone (0.02 to 0.99, no confidence); a
    choice `{type: "choice", choice, confidence, probabilities}`; a score `{type: "score",
    score, confidence, legend, probabilities}`, whose `score` is fractional in 5,516 of 5,740
    answers (an expected level such as 1.88, not a level), whose `probabilities` are keyed by the
    level as a string, and whose `legend` maps each level to `{summary, signals}`. `usage` holds
    `input_tokens` and `output_tokens`.
  - The other's working client: a noul's `criteria` is
    `{true, false}`, a choice's a map of option to description, and a score's an array of level
    descriptions; `state` may be an object; it retries 429, 503 and 529 with backoff and cuts an
    error body to 300 characters. Its type for `legend` is a map of strings, the recorded answers
    hold objects, so the parser keeps `legend` as JSON.
  - So: a score's reading is the fractional `score` with its `confidence`, `request::build` sends
    a score's levels as `criteria` (an array), 503 joins the retried statuses, and the parser
    ignores fields it does not know. The first real call is the layer's own check, once Chad sets
    his key.
- **For Complete:** eight queued specs (#566 to #573) link `queued/565-system-one-layer.spec.md`;
  they are pointed at `completed/` when this pair is archived.

### Design
- **Changed at promotion** (the seams re-read on 2026-09-27; each item below overrides the
  drafted design after it):
  - **The answer and the request** follow the recorded shape (the Promotion entry): a noul's
    `noul` alone; a choice's `choice`, `confidence` and `probabilities`; a score's fractional
    `score`, `confidence`, `probabilities` by level and `legend` kept as JSON. `request::build`
    sends a score's levels as `criteria`, an array. 503 joins 429 and 529 as a status retried once
    inside the deadline.
  - **HTTP** as #535's `push.rs` does it (`:90-96`, `:147-184`): `cx.http_client()` in a
    background task, a `Request` builder with the bearer header, `AsyncBody::from` the JSON,
    `send`; plus `HttpRequestExt::timeout(deadline)` (`http_client.rs:56-68`; reqwest applies it
    to the body too, `reqwest_client.rs:286-288`) raced with a background timer through
    `futures::future::select`, as `agents.rs:208` and `browser.rs:3770` race theirs. The body is
    read with `futures::AsyncReadExt::read_to_string`, as `anthropic.rs:525` reads an error. The
    endpoint check reuses `push.rs`'s loopback test (`target_url`, `:107-132`) in shape.
  - **The mask.** `mcp.rs::agent_redactor` (`:377`) answers `None` while
    `redact_secrets_for_agents` is false (`:380`), and D4 wants masked text whatever that setting
    says, so `mcp.rs` gains an accessor that answers the global's redactor on or off (the built-in
    rules alone before `start`). The key's own value is replaced by `[redacted: secret]` after the
    redactor: #516's `secret` rule does not match `MARLEY_SYSTEM_ONE_KEY=` (`_KEY` alone names no
    secret, `redact.rs` after #562). The marker is `[redacted: <kind>]` (`redact.rs:16-18`).
  - **The key.** gpui's calls are unchanged (`app.rs:1585`, `:1595`, `:1600`). On Linux they are
    `oo7` with the fixed label `zed-github-account` and the URL and username as attributes
    (`gpui_linux/src/linux/platform.rs:53`, `:719-766`), nothing depends on the channel, and
    `unlock()` may prompt. So the keychain is read only while `enabled` is true, the provider is
    `typesafe` or `compatible` and the variable is unset; read once, and again when the provider or
    endpoint changes or after Set key or Forget key. The dev box runs gnome-keyring and
    `script/e2e.sh` does not isolate the session bus, so a scenario reaches Chad's real keyring:
    no scenario presses Set key or Forget key, and every scenario exports the variable or leaves
    the layer off.
  - **Settings.** `SystemOneSettingsContent` nests like #535's `MarleyPushSettingsContent`
    (`marley.rs:52`, `#[with_fallible_options]`). `SystemOneProvider` and `SystemOneMode` live in
    `settings_content` beside `MarleyTerminalLinks` and `MarleyLayout` (`:62-111`), whose `strum`
    derives resolve only there; the pure crate keeps its own `Provider` and `Mode`, and the
    workbench maps one to the other. `uses` is a `BTreeMap<String, SystemOneMode>` (`MergeFrom`
    has `BTreeMap`, `merge_from.rs:89-135`, per key, so a default `check` can be overridden and
    not removed). The price is an `f32` of cents per million tokens in the content;
    `MarleySettings` (`marley_workbench.rs:203`, which derives `Eq`) holds it as an integer of
    thousandths of a cent, and the gate counts spend in billionths of a cent.
  - **The page.** `marley_page.rs` gains `system_one_section` after `push_section` (`:170`),
    with a nested text field like `marley.push.url` (`:173-197`) for Endpoint and Model, a `u64`
    field like `:70-91` for Daily Budget, two dropdowns (their renderers added to
    `settings_ui.rs:558-561`, a fifth Zed path, ledger row 60), a map-keyed item for the check's
    mode (`pick` reads `uses.get("check")`, `write` inserts; `page_data.rs:9267` is the map-keyed
    precedent), and an `ActionLink` (`settings_ui.rs:1703`) that dispatches `marley::OpenDecisions`
    by name through `App::build_action` (`app.rs:2396`) into the original window, as Open Keymap
    does (`page_data.rs:1610`), since `settings_ui` depends on no Marley crate.
  - **The check.** `LastTerminal` (`browser.rs:7378-7383`) is private; `browser.rs` gains a
    crate-visible accessor for the view. The project is `TerminalView::marley_workspace()`
    (`terminal_view.rs:1028`), the title `tab_content_text`, the last block `blocks().last()`
    (`terminal.rs:1915`; `AnchoredBlock` at `anchored.rs:38`, `exit_code: ExitCode(Option<i32>)`).
    No block records a program, and `foreground_process_command_name` names the shell once a
    block ends, so the program is the command's first word after its `NAME=value` assignments,
    masked like text. The command is masked whole, then cut to 300 characters (`pick::within`,
    `pick.rs:621`).
  - **The view.** No Marley workspace item exists but the Browser tab (#524's is a modal). The
    view follows Zed's `KeyContextView` (`language_tools/src/key_context_view.rs:135`) for the
    item's shape and `TelemetryLogView` (`crates/zed/src/zed/telemetry_log.rs:61`, `:483`) for a
    list of logged events, both read and none of their bodies copied (§20). Its header's first
    button is Run Check, which dispatches the check.
  - **Manifests.** `marley_workbench/Cargo.toml` has `http_client` already (#535); it gains
    `env_var` and `marley_system_one`. The pure crate takes `serde`, `serde_json`, `sha2` and
    `chrono` (the day's file name from a date it is given). Root `Cargo.toml`: members `140-148`
    (nine Marley crates, ten with this one), the Marley `[workspace.dependencies]` at `421-428`.
  - **The scenario** reuses #535's fake HTTP server (`535-phone-push-notifications.sh:117-140`)
    and #516's `marley_setting` (`516-secret-redaction-for-agents.sh:107`) for the live changes.
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
  `marley_workbench.rs` (modules, the two actions, `MarleySettings`, `init`), `mcp.rs` (the
  redactor accessor), `browser.rs` (the `LastTerminal` accessor), `Cargo.toml`
  (`marley_system_one`, `env_var`). Zed paths: root `Cargo.toml`,
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `crates/settings_ui/src/settings_ui.rs`, `assets/settings/default.json`. Generated: `Cargo.lock`. Scripts:
  `script/e2e/565-system-one-layer.sh`. Docs at Complete:
  `docs/marley_architecture/marley_system_one.md` (new), `marley_workbench.md`.
- **Ledger rows** (`docs/marley/zed-touchpoints.md`, written before the Code step that needs
  them): the `Cargo.toml` row names ten members and `marley_system_one` among the workspace
  dependencies; the `marley.rs` row gains the `system_one` block and its enums; the
  `marley_page.rs` row gains the System One section; the `settings_ui.rs` row (60) gains the two
  enums' dropdown renderers; the `default.json` row gains the block's defaults.

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

Not reachable by a scenario: a real TypeSafe answer (the fake speaks the shape Chad's recorded
answers show; the first real call is the layer's own check once Chad sets his key), the keyring
path (the headless run reaches Chad's own gnome-keyring over his session bus, so no scenario
writes or reads it: REQ-009's keyring half is proven by review of the read and write paths, the
environment path by the scenario), and a day boundary for the budget (reviewed: the day is the
local date of each row).

### Risks
- The answer shape is the reference as read on 2026-09-26, and the SDKs may add fields; the
  parser ignores unknown fields, and the promotion's live request settles it before code.
- The keyring: gpui's Linux keychain needs a Secret Service (gnome-keyring on the dev box), and
  its `unlock()` may prompt. The layer reads it only while on and with no variable set; without
  a service the read fails and the source reads `none` with the reason. A scenario reaches the
  user's own keyring, so none presses Set key or Forget key.
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
- **Checklist** (no task tool): the ledger rows ✓ (46, 55, 58, 60, 61 widened first); the crate ✓;
  root `Cargo.toml` ✓; `marley.rs`, `settings_ui.rs`, `marley_page.rs`, `default.json` ✓;
  `system_one.rs`, `decisions.rs`, `mcp.rs`, `browser.rs`, `marley_workbench.rs`, the manifest ✓;
  check, fmt and clippy ✓; the review ✓.
- **Built.**
  - `crates/marley_system_one` (MIT OR Apache-2.0, rustal's lint table, root
    `marley_system_one.rs`): the root holds `QuestionSet`, `Question` (noul, choice, score),
    `UseSpec`, `DEFAULT_MODEL` and the check's `CHECK_SET` (`check/1`, one noul,
    `command_failed`) and `CHECK` (a 2 s deadline). `state`: `Detail`, `StateBuilder` (every
    value through the host's mask; a text value masked whole, then cut to 300 characters, and left
    out at `Facts`), `State` with its SHA-256. `request`: `build` (a score's levels as `criteria`,
    an array), `Answer`, `Answers` (`by_key`, `unreadable`, `raw`, `input_tokens`), a tolerant
    `parse`, `error_excerpt`. `reading`: the floor and the noul band, `Reading` (`Off`, `Rules`,
    `Model`, `Refused`, `Unavailable`), `Read`, `Signal`, `summary` (`command failed 0.90`).
    `policy`: `may_send` (a remote project sends nothing; the metadata-only list wins), `cost` and
    `budget` in billionths of a cent, `Gate` (the day's spend, the breaker, 1,000 calls a minute,
    the repeat check). `files`: `CallRow`, `OutcomeRow`, `Row`, `append_in` (0600 files),
    `read_day_in`, `Replay` with `load_in` and `answer`.
  - Settings: `SystemOneSettingsContent`, `SystemOneProvider` and `SystemOneMode` in
    `settings_content/src/marley.rs`, the `system_one` block in `default.json` (off, `typesafe`,
    `jev-1.13.0`, no projects, 50 cents, the check at `act`), the two dropdown renderers in
    `settings_ui.rs`, and `system_one_section` on the Marley page (the switch, Provider, Endpoint,
    Model, Daily Budget, the check's mode, and an Open Decisions link that builds
    `marley::OpenDecisions` by name).
  - `marley_workbench::system_one`: `SystemOneSettings` (in `MarleySettings`, the price in
    thousandths of a cent through a rounded decimal, since a lossy cast is refused by the lint
    table), the `Key` newtype (`Debug` prints `Key(***)`; it leaves only as the header and as its
    own mask), `KeySource`, the `SystemOne` global, `init` (the settings observer and the two
    actions), `load_key` (only while on and for a provider that sends; the variable, else
    `read_credentials` at the endpoint, a newer load winning), `store_key` and `forget_key`,
    `load_replay` (off the main thread), `ask`, `send` (the endpoint check before the key is
    used, the gate, the request in a background task), `post` (the timeout extension raced with a
    backstop timer, one retry on 429, 503 or 529 while time remains), `read_posted` (an answer
    spends, a failure counts toward the breaker, 401 marks the key refused), `finish` and the log
    task, and the check (`run_check`, `check_asking`, `program`, the toast).
  - `marley_workbench::decisions`: `DecisionsView`, a workspace item: the header (calls today and
    the spend against the budget, the provider and model, the key's source, an open breaker), Run
    Check, Set Key (a masked single-line editor; Enter is `menu::Confirm`, Escape
    `editor::Cancel`), Forget Key, and the day's rows newest first, each opening to the state as
    sent, the answers and the error. It reads today's file when it opens and follows the global.
  - `mcp.rs`: `model_redactor`, the rules and patterns whatever agents' redaction says.
    `marley_mcp::redact::marker` is public. `browser.rs`: `last_terminal`.
- **Deviations from the plan.**
  - Six files in the crate, not eight: the question types and the check's set sit in the root,
    and the log and the replay in `files.rs` (Zed's `.rules`: few small files).
  - The crate keeps no `Mode` or `Provider`: the enums live once, in `settings_content`, and a
    row names them as strings.
  - A set's dynamic options (templates filled at call time) wait for #567, their first user.
    `OutcomeRow` is written by no use yet.
  - The palette names the view `marley: open decisions`; the header's first button is Run Check.
  - `Answers::answers` became `by_key` (clippy's field-name rule).
- **Checks.** `cargo check` on the four crates; `cargo fmt`; `just clippy marley_system_one
  marley_mcp marley_workbench settings_content settings_ui` (all targets, `-D warnings`) clean
  after five rounds: doc lines naming TypeSafe without backticks (reworded), `Eq` beside
  `PartialEq` on three types, the log's large variant (boxed), a `const fn`, the struct-named
  field, unused `self` twice, three `&mut` used as `&`, two arguments that borrow, `map_or_else`,
  `is_multiple_of`, a trailing semicolon, and `buffer_font`'s missing context.
- **Review of the diff.**
  - REQ-001: with the layer off, `ask` answers `Off` before anything else, `load_key` reads
    nothing, the log task never starts, and `folder()` is made only by the first row.
  - REQ-002 to REQ-008: the paths above; the budget refuses at a spend of the cap, so a cap of 0
    refuses the first call; the breaker opens on the fifth failure in a row and closes after two
    minutes.
  - REQ-009: the key's value is in the global and in the one background task's header, masked out
    of the state and out of every error kept; no `Debug` or row carries it.
  - Re-entrancy: **found and fixed**. `run_check` runs inside the Workspace's update, and
    `check_asking` read the terminal's workspace with `read(cx)`: for a terminal of that same
    workspace, the usual case, gpui panics. The check now takes the folders from the `&mut
    Workspace` it has when the terminal is that workspace's, and reads another workspace only when
    it is not (an `F-` block at Complete).
  - Provenance: nothing of Warp's; no Zed function body copied (Zed's `KeyContextView` and
    `TelemetryLogView` read for the item's shape); Chad's own Jev client read for the
    request's shape; `s1-rs` not read.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario ✓; run and read every shot ✓ (two runs); 515 again
  ✓; the golden set with 565 ✓; the gate ✓.
- **The scenario.** `script/e2e/565-system-one-layer.sh`, `compositor sway`, so nothing reaches
  Chad's session and no focus report applies. A fake `/v1/systemone` (Python, a free loopback
  port) logs each request's path, headers and body and answers by the state: `slow` sleeps 3 s,
  `fail` answers 500, anything else `command_failed` at 0.92, or 0.08 when the state says
  `exit code: 0`, with 1,200 input tokens. `setup` exports `MARLEY_SYSTEM_ONE_KEY` (a test key,
  never printed) before Marley starts and turns the layer on (`compatible` at the fake's URL, the
  scratch repository listed) through `system_one_setting`, which rewrites the profile copy's
  settings as JSON; every later change goes through it while Marley runs (REQ-011). Each check is
  a command typed into the terminal from its rail row, then `marley: system one check` from the
  palette. No step presses Set Key or Forget Key: the headless run reaches Chad's own keyring.
- **Deviations from the E2E plan.**
  - The fake answers by the exit code, so `echo hello` reads `command failed: no (0.08)` and
    `false` `command failed: yes (0.92)`.
  - A slow answer counts toward the breaker, so an `echo ok` check that answers comes between it
    and the five failures; the breaker's check follows them.
  - Decisions opens twice: after the first two checks (the key's source, a row opened), and after
    the replay, with every call, the masked and the metadata-only rows opened in turn.
  - The Settings window opens last, with the layer back on at `compatible`, since it narrows the
    main window (L-claude-516).
- **Run 1:** every check passed and every shot showed its criterion; two changes before run 2. A
  noul that did not hold read `not command failed 0.08`, which is not English: a noul now reads
  `command failed: yes (0.92)` or `command failed: no (0.08)`. The masked and metadata-only toasts
  cannot show what was sent, so run 2 opens those rows in Decisions; and the Settings shot scrolls
  to the end of the section with the layer back on.
- **Run 2: every check passes, 18 of 18.** The shots, in `scratchpad/565/run2`, each read:
  - `565-02a-answered` (REQ-002): the toast `System One: command failed: no (0.08) · compatible ·
    1,200 tokens · 26 ms` after `echo hello`.
  - `565-02b-failed`: `command failed: yes (0.92) · compatible · 1,200 tokens · 33 ms` after
    `false`. The log: two requests, each with the bearer header and `jev-1.13.0`, the state holding
    `project: repo`, `exit code: 1`, `program: false` and `last command: false`.
  - `565-02-decisions-view` (REQ-002, REQ-009): `2 calls today · 0¢ spent of the 50¢ budget` (a
    compatible server's price is 0), `Provider: compatible · jev-1.13.0`, `Key: environment
    (MARLEY_SYSTEM_ONE_KEY)`, Run Check, Set Key, Forget Key and the note that the variable comes
    before the keyring; the two rows, newest first. `565-02c-expanded`: the `false` row open on the
    state as sent (project, block 1, exit code 1, program, terminal, last command) and the answer.
  - `565-03-refused-not-listed` (REQ-003): with `projects` emptied, `System One: Refused: project
    not listed`; the fake had no new request.
  - `565-04-masked`, `565-04b-masked-row` (REQ-004): the row's state reads `last command: echo
    GITHUB_TOKEN=[redacted: secret]`; the fake's body holds the marker and no `FakeFake`.
  - `565-05-metadata-only`, `565-05b-metadata-only-row` (REQ-005): the row's state is `project`,
    `block`, `exit code: 0` and `program: echo`, no terminal title and no command; the body holds no
    `hidden`.
  - `565-06a-slow` (REQ-006): `Unavailable: no answer within 2 s · 2000 ms`. `565-06b-failed`:
    `Unavailable: the provider answered 500 · 36 ms`, the fifth failure. `565-06c-breaker-open`:
    `Unavailable: breaker open`; the fake had 11 requests before it and none from it.
  - `565-07-budget` (REQ-007): with a budget of 0, `Unavailable: over the daily budget`, no
    request.
  - `565-08-replay` (REQ-008): `command failed: yes (0.77) · replay`, no request; the day's last row
    names `replay`.
  - `565-08b-decisions-all`: `15 calls today`, `Provider: replay`, `Key: not needed`, the red line
    of the open breaker, and every call newest first, the refused and unavailable ones in red; the
    day's file holds 15 calls.
  - `565-09-off` (REQ-001): with `enabled` false, `System One is off. Turn it on in the System One
    section of the Marley settings.`; no request and no new row.
  - `565-01a-marley-page`, `565-01-settings-section` (REQ-010): the Settings window's Marley page,
    and scrolled, the System One section: the switch on, Provider Compatible, the endpoint, Model
    `jev-1.13.0`, Daily Budget 50, Check Act, and Decisions with Open Decisions.
  - REQ-009: `grep -rF` finds the key's value nowhere under the profile, Marley's log included.
- **515 again:** passes; its shots show the Marley page and the switch to Zed's layout, the new
  section below the fold.
- **The golden set** (38 with 565): all 38 pass, 565 in 214 s.
- **The gate** (`script/gates.sh --diff`, the logs in `scratchpad/565/gate.log` and `gate2.log`).
  The first run was red at gate:14, rustdoc: the public modules' docs linked to the crate-private
  `SystemOne` and `ask` (`rustdoc::private_intra_doc_links`); the links are plain code now. The
  second run: every gate PASS, `GATE GREEN [diff]`, and the receipt written. Only two doc comments
  changed since run 2, so its shots stand for the tree as committed.
- **Not reached by a scenario:** a real TypeSafe answer (the fake speaks the recorded shape; the
  first real call is the check once Chad sets his key); the keyring's read and write (the run
  reaches Chad's own keyring; reviewed); a day boundary for the budget (reviewed: the day is each
  row's local date).
- **Verdict: PASS.** Every criterion shows in a shot or the log, REQ-012's gate and golden set
  included.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md` (Added: a System One layer, off until you turn it on). The new
  crate note `docs/marley_architecture/marley_system_one.md`, listed in `docs/marley/README.md`;
  `marley_workbench.md` (the System One section: the settings, the global, the key, `ask`, `post`,
  the log task, the check and Decisions; `model_redactor` in the MCP server's redaction);
  `marley_mcp.md` (`marker` public). The plan: an `S1` row in prong 2's slices. The guide: a System
  One section (turning it on, what leaves the machine, the check, Decisions, the providers, the
  budget and the failures), its line in the contents and in What Marley adds, the key's variable,
  and the `system_one/` folder. The five ledger rows (46, 55, 58, 60, 61) describe what shipped.
- **Knowledge appended:** F-claude-565-the-check-read-its-workspace-inside-that-workspaces-update-001
  (under the existing PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001),
  L-claude-565-jevs-answer-shape-as-recorded-001, L-claude-565-a-poll-in-setup-dies-under-set-e-001,
  L-claude-565-a-scenario-reaches-the-users-own-keyring-001,
  AD-claude-565-the-system-one-layer-is-a-pure-core-behind-an-adapter-off-by-default-001. The
  brain: consultation de2f88036a50474db9ab866c2a613c28 closed with
  `decisions/marleys-system-one-layer-a-pure-core-behind-a-workbench-adapter-off-by-default-marley-565`,
  follow-up by 2026-10-27.
- **Closed** TICKET-565; its backlog row left at promotion. The eight queued specs that named
  `queued/565-system-one-layer.spec.md` name `completed/` now.
