---
pipeline_id: 81af5545-1e45-4030-a1ee-a5864f750b9d
ticket: docs/planning/tickets/closed/TICKET-565-system-one-layer.md
status: Phase 4 — Complete PASS
title: "The System One layer: typed decisions, off by default"
type: feature
slice: prong 2 (the control plane); the Jev note's use 0, the layer #566, #567, #568 and #548 build on
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/tickets/open/TICKET-548-system-one-via-cloudflare.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/515-marley-settings-page.spec.md, docs/marley/three-prong-plan.md]
---

## Title
A `marley_system_one` crate asks a System One model typed questions (a noul, a choice, a score)
about a state Marley builds from facts computed in code, and hands each use a reading it may
display, rank, route on or refuse with, never an approval. Jev through TypeSafe's API is the first
provider; any server speaking the same `/v1/systemone` request is the second; `rules` answers
from each use's own deterministic verdict, and `replay` from a file, which is what every use's
e2e scenario runs on. Every call is a masked JSON line under Marley's data directory, a Decisions
view shows the day's calls beside the day's spend, and `marley.system_one` holds the switch,
the provider, the pinned model, the project allow list, each use's mode and a daily budget. The
key comes from `MARLEY_SYSTEM_ONE_KEY` or the keyring; its source shows, its value never does.
Off by default: a user who leaves it off sees nothing new, and nothing in Marley requires a
model to work.

## Scope
### In
- **`crates/marley_system_one`** (new, Marley-owned, `MIT OR Apache-2.0`, rustal's lint table,
  library root `src/marley_system_one.rs`, pure: no gpui, no HTTP, no clock of its own):
  - `question`: `QuestionSet { id, model, questions }` compiled in, named and versioned
    (`check/1`, `stop_kind/1`), insert-only: a change to a question is a new version. A
    `Question` is `Noul { key, instructions, criteria: (true, false) }`, `Choice { key,
    instructions, options: [(name, description)] }` (at most 255 options, and a `cannot_tell` or
    `none` option wherever the state may not settle it) or `Score { key, instructions, levels }`
    (2 to 10, sent as `criteria`, an array of the levels' descriptions). A set with dynamic options (a page's refs, a prompt's parts) is filled from a
    template at call time; the set's id and version stay. A caller never supplies its own set.
  - `state`: `StateBuilder` takes labeled facts (computed in code: names, counts, exit codes,
    states) and labeled text (the prompt, a message, a command), each text value through a
    `Mask` the host supplies (#516's redactor), and renders one plain-text state of labeled lines
    plus its SHA-256. A `Detail` of `Full` or `Facts` (the metadata-only mode) decides whether
    the text lines are included. A state empty by design is a refusal, not a request.
  - `request`: `build(model, state, set) -> serde_json::Value` in the `/v1/systemone` shape
    (`{model, state, questions: {key: {type, instructions, criteria}}}`) and
    `parse(body) -> Result<Answers, ParseError>` for the answer shape as Jev answers it (the
    notes' Promotion entry): per key `{type: "noul", noul}`, `{type: "choice", choice,
    confidence, probabilities}` or `{type: "score", score, confidence, probabilities, legend}`,
    where a score's `score` is fractional (the expected level), its `probabilities` are keyed by
    the level, and `legend` is kept as JSON; then `model` and `usage.input_tokens`. Fields it
    does not know are ignored. An error body is cut to 300 characters before it is kept anywhere.
  - `reading`: thresholds per question (a choice or a score needs `confidence` at or above 0.5
    and an option other than `cannot_tell` or `none`; a noul inside 0.35 to 0.65, widened by 1e-9,
    is no signal), and `Reading`: `Off`, `Rules(verdict)`, `Model { answer, confidence, call }`,
    `NoSignal { reason, call }`, `Unavailable { reason }`, `Refused { reason }`. A shadow
    reading is wrapped as `Shadow(reading)`, which a use can log and not show. Nothing here is an
    `Err`: model trouble reaches a use as `Unavailable` with its reason.
  - `policy`: `may_send(project_folders, allow_list, metadata_only) -> Detail | Refused`;
    `Budget { daily_cap_cents, spent_today }`, the spend counted in billionths of a cent, since a
    call of 1,200 tokens costs about 0.005 cents; `Breaker` (open for two minutes after five
    failures in a row); `Bucket` (1,000 requests a minute); `dedupe(subject, state_hash)` (the same
    masked state as the last one asked for a subject makes no new call); cost from
    `input_tokens` and the model's price in thousandths of a cent per million tokens (`jev-*`:
    4,200, TypeSafe's 0.042 USD; a `compatible` server's from settings, 0 unless set).
  - `log`: a `CallRow` (id, time, use, set, model, provider, project name, mode, the use's own
    verdict, the masked state as sent, the question keys, the raw answers, the reading, the
    thresholds, latency, tokens, cost, error) and an `OutcomeRow` naming a call id; `append_in(dir,
    row)` writes one line to `<dir>/calls-YYYY-MM-DD.jsonl`; no row is rewritten.
  - `replay`: rows `{set, match?, state_hash?, answers, repeat?}` read from
    `<dir>/replay.jsonl`; the first row whose set matches and whose `match` (a substring of the
    masked state) or `state_hash` fits answers, consumed unless `repeat`; no row is
    `NoSignal("no replay row")`, logged like any call.
- **`crates/marley_workbench/src/system_one.rs`** (new): the `SystemOne` global: the settings as
  resolved, the key and its source (a manual `Debug` that never prints it, no getter), the
  budget, the breaker, the bucket, the dedupe table, the log writer (a background task the rows
  are sent to), and `ask(use, state, verdict, cx) -> Task<Reading>`: refuses by policy, dedupes,
  then for `typesafe` and `compatible` builds the request, sets `Authorization: Bearer` and the
  use's deadline (`HttpRequestExt::timeout` and a background timer as the backstop), sends it
  through `cx.http_client()`, parses, reads, logs, and answers; one retry on 429, 503 or 529
  inside the deadline; 401 marks the key refused until the key changes. The state is masked with
  #516's rules and the user's patterns whatever `redact_secrets_for_agents` says (an accessor in
  `mcp.rs` that answers the redactor even while agents' redaction is off), and the key's own
  value is masked too, since #516's `secret` rule does not name `MARLEY_SYSTEM_ONE_KEY`. `rules` answers the use's own verdict
  and logs it; `replay` answers from the file. `UseSpec { name, set, deadline }` per use; this
  ticket registers `check`.
- **The key.** `MARLEY_SYSTEM_ONE_KEY` (an `env_var::EnvVar`, empty is none) first; else gpui's
  own keychain (`cx.read_credentials` at the provider's URL, username `system-one`), never Zed's
  dev-channel credentials file. The keychain is read only while `enabled` is true and the
  provider needs a key, so a user who leaves the layer off never meets an unlock prompt. On
  Linux it is the Secret Service through `oo7`, one fixed label for every item and the URL and
  username as attributes. The Decisions view offers "Set key" (a masked single-line
  editor; Enter writes the keyring and clears the editor) and "Forget key". The key is a request
  header only; the endpoint must be `https`, or `http` on a loopback address for `compatible`,
  checked in code before the header is set.
- **Settings.** `marley.system_one` in `MarleySettingsContent` (a nested
  `SystemOneSettingsContent`): `enabled` (false), `provider` (`typesafe` | `compatible` | `rules`
  | `replay`; #548 adds `cloudflare`), `endpoint` (for `compatible`), `model` (`jev-1.13.0`),
  `projects` (folders that may send state; none), `metadata_only_projects` (none),
  `daily_budget_cents` (50), `price_cents_per_million_tokens` (for `compatible`; 0), and `uses`
  (a `BTreeMap` from a use's name to its mode: `off` | `shadow` | `suggest` | `act`; the settings'
  maps merge key by key). The two enums (`SystemOneProvider`, `SystemOneMode`) live in
  `settings_content` with the `strum` derives the page's dropdowns need; the price is an `f32` in
  the content and an integer number of thousandths of a cent in `MarleySettings`, which stays
  `Eq`. `MarleySettings` resolves them; `default.json` gains the block; the Marley settings page
  gains a System One section: Enabled, Provider, Endpoint, Model, Daily Budget (cents), the
  check's mode, and an action link "Open Decisions" whose description says the lists live in
  `settings.json` and that the Decisions view shows the key's source. `settings_ui.rs` registers
  the two enums' dropdown renderers, and the link dispatches `marley::OpenDecisions` by name,
  since `settings_ui` depends on no Marley crate.
- **The check** (`marley: system one check`, the `check` use, `check/1`): the state of the
  terminal the user focused last (`LastTerminal`, given a crate-visible accessor): the project's
  name, the terminal's title and the last block's command, masked whole and then cut to 300
  characters, with its exit code and program as facts (the program is the command's first word
  after any `NAME=value` assignments, masked like any text, so `GITHUB_TOKEN=… git push` is
  `git`); one noul,
  `command_failed`. Its reading shows as a toast (`System One: command failed 0.90 · compatible ·
  1,200 tokens · 310 ms`) and in the Decisions view. It runs only by hand, so its default mode is
  `act`; `off` still turns it off.
- **The Decisions view** (`marley: decisions`, `crates/marley_workbench/src/decisions.rs`, a
  workspace item): a header with the day's calls and spend against the budget, the key's source
  (`environment (MARLEY_SYSTEM_ONE_KEY)`, `keyring`, `none`), the provider and model, the
  breaker's state, "Run Check" (the check, as the palette runs it), "Set key" and "Forget key";
  then the day's rows, newest
  first: time, use, project, provider and model, the reading with its number (`command failed
  0.90`, `would show: …` in shadow, `Refused: project not listed`, `Unavailable: …` in red),
  latency, tokens and cost; a row expands to the masked state as sent and every probability. It
  reads the day's file when it opens and follows the global for new rows. Zed's
  `TelemetryLogView` is the nearest shape (a list of logged events); nothing of its body is
  copied (§20).
- **The scenario** `script/e2e/565-system-one-layer.sh`, with a fake `/v1/systemone` server.

### Out (explicitly deferred)
- The uses: the stop kind (#566), the find tools (#567), the inbox order and chips (#568), and
  the rest of the note's ranked list. Cloudflare Workers AI and a provider per project (#548).
- Fitted thresholds from labels, `script/system-one-eval.sh` and the golden report, and manual
  labels in the Decisions view (a "correct this" control); the compiled-in thresholds stand until
  a use has labels.
- The coach-and-executor playbook (versioned rule lists, the validator, sealing), the harness
  decider (M10) and `session_answer`.
- A local model (Laya, Reflex) as a provider; an `openrouter` provider of its own (an endpoint
  that speaks the same request is `compatible`).
- A status bar item for the day's calls; `notifications/tools/list_changed`; pruning old log
  files; a per-use provider override.
- Encrypting the log: it holds masked state on Chad's own disk, as Marley's other data does.

## Reference (§20)
N/A — Marley-specific: no reference product has a typed-decision layer. Warp's agent platform
(the agent notifications and mailbox #508 and #519 cite from docs.warp.dev) makes its judgments
inside its own model prompts and documents no decision model, and nothing of Warp's was read;
Orca (`docs/orca_architecture/`) reports agent state from hooks and decides nothing. The closest
upstream Zed shape is `language_models`: a provider with an API key from an environment variable
or the keychain (`crates/language_model/src/api_key.rs`), whose key handling the layer follows.
The behavior Marley matches is TypeSafe's published API and patterns and Chad's own ground rules
for using Jev, as the note records them.

### Prior art
- **Behavior maps.** `docs/warp_architecture/subsystems/04-agent-ai-mcp.md` (Warp's agent and
  MCP layer: nothing decides on a typed model); `docs/orca_architecture/01-agents-and-sessions.md`
  (hook-driven state, no decisions); the three-prong plan's D7 (`marley_fleet` is the envelope)
  and D8 (one adapter crate per service, a pure core plus a thin transport), which this crate
  follows; the Jev note's decision layer, budget and safety rules, and Chad's answers of
  2026-09-26.
- **Published material.** TypeSafe's API reference (docs.typesafe.ai/api, read 2026-09-26): `POST
  https://api.typesafe.ai/v1/systemone` with `Authorization: Bearer`, a request of `model`,
  `state` (text or structured) and `questions` (a map of named questions: `noul` with
  `instructions` and optional `criteria` `{true, false}`; `choice` with a `criteria` map of at
  most 255 options; `score` with 2 to 10 ordered levels), an answer of `model`, `answers` (per key:
  `type` and `noul`; or `choice`, `probabilities`, `confidence`; or `score`, `legend`,
  `probabilities`, `confidence`) and `usage` (`input_tokens`, `output_tokens`); 401, 422, 429 and
  529 with backoff. The models page: `jev-1.13.0` behind `jev-latest` and `jev-preview`, "pin
  that version's ID". Confidence routing (docs.typesafe.ai/patterns/confidence-routing: a 0.6
  floor and a 0.85 act line as starting points); the consistency cookbooks (abstaining under 0.60
  lifted agreement from 90.8% to 99.2%); the model jaggedness page (reads text only, counts badly,
  cannot say "don't know" without an option). The independent tests the note gathered (beri.net's
  calibration study, JevBench v1.2, the action-gate study, jev-certify, the KoBBQ audit,
  OpenRouter's 791 calls) set the thresholds' starting points and the per-question rule. The
  community list (github.com/AbdelStark/awesome-typesafe-jev): Rust clients `s1-rs`
  (github.com/AbdelStark/s1-rs: typed Choice, Score and Noul, question sets, confidence gates,
  network-free testing), `typesafe-rs` and `Twister915/typesafe-ai`; and Jeview (a loopback proxy
  that stores calls locally), jevcal (thresholds fitted to labels) and Second Thought (captured
  decisions and calibration). Read for their shapes; none is linked: `s1-rs` needs tokio, and
  `marley_mcp` set the precedent of no SDK that brings a runtime into a gpui app.
- **The code we already ship.** Zed's `http_client`: `HttpClient::send`
  (`crates/http_client/src/http_client.rs:123-131`), `HttpRequestExt::timeout` and
  `RequestTimeout` (`:31`, `:56-67`, "a deadline for the complete HTTP request, including its
  response body"), `cx.http_client()` (`crates/gpui/src/app.rs:1740`); the `anthropic` crate's
  POST idiom (`crates/anthropic/src/anthropic.rs:496-520`: a builder, the key header,
  `AsyncBody::from(serialized)`, `client.send`). Keys: `ApiKeyState`
  (`crates/language_model/src/api_key.rs:19`; `load_if_needed` at `:160` takes a non-empty
  environment variable first, else the keychain; `ApiKey::from_env` at `:233`), `env_var::EnvVar`
  (`crates/env_var/src/env_var.rs:11`, an empty value is none), gpui's `write_credentials`,
  `read_credentials` and `delete_credentials` (`crates/gpui/src/app.rs:1585-1601`; on Linux
  `gpui_linux` reaches the secret service through `oo7`, `Cargo.lock`), and
  `zed_credentials_provider` (`:22`, `:45-64`: the dev channel keeps credentials in a
  development provider unless `ZED_DEVELOPMENT_USE_KEYCHAIN` is set, which is why the layer calls
  gpui's keychain itself). Masking: `marley_mcp::redact::Redactor` (`crates/marley_mcp/src/redact.rs:126`,
  `redact` at `:152`) and `mcp::agent_redactor` with its settings observer
  (`crates/marley_workbench/src/mcp.rs:267`, `:297`). Settings: `MarleySettingsContent`
  (`crates/settings_content/src/marley.rs:10`; `MarleyLayout`'s `strum` derives at `:33-46` give a
  dropdown), `MarleySettings` and `from_settings` (`crates/marley_workbench/src/marley_workbench.rs:168-197`),
  the Marley page (`crates/settings_ui/src/marley_page.rs:6`, `:42`) and `settings_ui`'s item
  kinds (`SettingsPageItem` at `crates/settings_ui/src/settings_ui.rs:1087`; `ActionLink` at
  `:1701`). Files: `paths::data_dir` (`crates/paths/src/paths.rs:145`; `--user-data-dir` sets it,
  `crates/zed/src/main.rs:263`) and the browser's folder under it (`crates/marley_workbench/src/browser.rs:627`).
  The view: `impl Item for BrowserView` (`browser.rs:4788`) and `add_item_to_active_pane`
  (`:5748`); `LastTerminal` (`:5400`); `Editor::set_masked` (`crates/editor/src/editor.rs:9012`);
  toasts (`marley_workbench.rs:430`, `mcp.rs:247`); background timers
  (`crates/marley_workbench/src/browser_tools.rs:137`). `sha2`, `chrono`, `serde_json`,
  `base64` and `futures` are workspace dependencies (`Cargo.toml:858`, `:599`, `:849`, `:588`,
  `:649`). Nothing in `Cargo.lock` speaks System One (no `s1-rs`, `keyring` or
  `secret-service`; `oo7` is gpui's). Does a crate we build own the seam? None owns typed
  decisions; `http_client` owns the transport, gpui the keychain, `marley_mcp` the mask,
  `settings` the configuration and `workspace` the view, and the layer takes each.
- **Re-verified at promotion (2026-09-27).** Chad's own Jev work gives the real answer shape:
  the recorded runs of one of his projects (922 answers, 574 from `jev-1.13.0`) and the working
  client in another (a score's `criteria` as an array, the
  retried 429, 503 and 529, error bodies cut to 300 characters), read for their structure only.
  #535's `push.rs` is now the nearest code: a bearer POST through `cx.http_client()` in a
  background task, and a loopback check on the target (`target_url`). The timeout extension
  covers the response body (`reqwest_client.rs`, its test of a slow body), and dropping the
  send cancels it. gpui's Linux keychain is `oo7` with one fixed label; the dev box runs
  gnome-keyring, which a scenario reaches, so no scenario writes it. `agent_redactor` answers
  nothing while agents' redaction is off, so the layer needs its own accessor. The settings'
  maps merge through `MergeFrom`, which `BTreeMap` has; the dropdown renderers are registered in
  `settings_ui.rs`, a fifth Zed path. `LastTerminal` is private to `browser.rs`, and a block
  records no program. The notes' Promotion entry has the file and line of each.

## UI proof
UI-AFFECTING, all of it opened on purpose: the Marley settings page's System One section, the
Decisions view and the check's toast. `script/e2e/565-system-one-layer.sh` (`compositor sway`:
the rail's terminal row and the view's rows are clicked). Setup starts a fake `/v1/systemone`
server (Python's `http.server` on `127.0.0.1`, a free port) that appends each request's headers
and body to `$E2E_WORK/systemone.log` and answers by the state's text: a state holding `slow`
sleeps three seconds, `fail` answers 500, and anything else answers `command_failed` at 0.92, or
0.08 when the state says `exit code: 0`, with `usage.input_tokens` 1,200. The profile's settings
enable the layer on `compatible` at the fake's URL with the scratch repository listed;
`MARLEY_SYSTEM_ONE_KEY=e2e-not-a-real-key` is exported before the launch, and no step presses Set
Key or Forget Key, which would write the user's own keyring. Each check is a command in the
terminal, then `marley: system one check` from the palette. Shots: `565-02a-answered` and
`565-02b-failed` (the toasts after `echo hello` and `false`: `command failed: no (0.08)` and
`command failed: yes (0.92)`, with the provider, tokens and latency); `565-02-decisions-view` (the
two rows, the day's spend against the budget, `Key: environment (MARLEY_SYSTEM_ONE_KEY)`) and
`565-02c-expanded` (a row opened to the state as sent); `565-03-refused-not-listed` (the project
removed from the list while Marley runs: `Refused: project not listed`); `565-04-masked` and
`565-04b-masked-row` (a command holding a token put together at run time: the row's state shows
`[redacted: secret]`); `565-05-metadata-only` and `565-05b-metadata-only-row` (the project on the
metadata-only list: the row's state holds the exit code and the program and no command);
`565-06a-slow` (`Unavailable: no answer within 2 s`), `565-06b-failed` (the fifth `echo fail`:
`Unavailable: the provider answered 500`) and `565-06c-breaker-open` (the sixth: `Unavailable:
breaker open`); `565-07-budget` (the cap at 0: `Unavailable: over the daily budget`);
`565-08-replay` (provider `replay` with a row the scenario wrote: the answer with `replay`) and
`565-08b-decisions-all` (every call, the refused and unavailable ones in red, the open breaker);
`565-09-off` (`enabled` false: the toast says the layer is off); `565-01a-marley-page` and
`565-01-settings-section` (the page scrolled to Enabled on, Provider Compatible, Endpoint, Model
`jev-1.13.0`, Daily Budget 50, the check's mode, and the Open Decisions link; no key anywhere).
Machine checks: the fake's log for the bearer header, `jev-1.13.0`, the state's facts, the masked
marker and no token, the metadata-only body without the command, and the count of requests after
each refused, breaker, budget, replay and off case; `grep` finds the key's value nowhere under the
profile, Marley's log included; the day's file has one row per check made while on, 15 in all.

## Locked-In Decisions
- D1: "Local first and then jev second" (Chad, 2026-09-26): a use hands the layer its own
  deterministic verdict with every ask, the `rules` provider answers with that verdict and
  nothing else, and a reading is a refinement a use may add to what its code already decided,
  never a replacement. The layer has no verb that approves, sends, types or stops anything.
- D2: Nothing in Marley requires a model (Chad: "Otherwise this becomes a jev required
  system"). The layer ships off, each use ships off with its own mode, and with the layer off or
  the provider down every use behaves as it does today; a missing key, a dead endpoint or a spent
  budget is an `Unavailable` reading with its reason, never an error a feature has to handle.
- D3: Provider-agnostic: the request shape is TypeSafe's published `/v1/systemone`, and
  `compatible` takes any endpoint that speaks it; `rules` and `replay` need no network. #548 adds
  Cloudflare as a fifth value of the same setting.
- D4: What leaves the box is decided in code before any request: a state is built only from
  facts and from text the redactor has masked, only for projects on the allow list, and for a
  metadata-only project from facts alone; a remote project sends nothing. The mask is #516's
  rules and the user's patterns, on whatever agents' redaction is set to, plus the key's own
  value. The masked state is what the log keeps.
- D5: Question sets are compiled in, versioned and insert-only, with the model id pinned per set
  (TypeSafe: "pin that version's ID"); a threshold belongs to one question, set version and
  model; the compiled-in starting points are the note's (a 0.5 confidence floor, the 0.35 to 0.65
  noul band, a `cannot_tell` option wherever the state may not settle a choice).
- D6: Every call is a row, failures included, so an unreachable model and a model that found
  nothing never look alike; outcomes arrive as later lines naming the call; no row is rewritten.
  The log lives under `<data>/system_one/`, never in a project.
- D7: The key is read from `MARLEY_SYSTEM_ONE_KEY`, else from gpui's own keychain at the
  provider's URL, and only while the layer is on; never from settings and never from the dev
  channel's credentials file. It is a
  header only; its source is shown, its value is not, and error bodies are cut to 300 characters
  because they can echo a request.
- D8: The budget is a daily cap in cents, spent by `input_tokens` at the model's price; one
  bucket keeps Marley under TypeSafe's 1,200 requests a minute; a breaker opens for two minutes
  after five failures in a row; retries happen only inside a use's deadline; a state that hashes
  the same as the subject's last one makes no call.
- D9: A pure core and thin adapters (plan D8, the house pattern): the crate knows no gpui, no
  HTTP and no clock; the workbench module owns the network, the files, the settings and the
  keychain, and file IO takes its directory (§14).
- D10: The check runs only by hand, so it is the one use whose default mode is `act`: it cannot
  spend on its own, and Chad needs a way to see one request and one answer without turning a use
  on.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.system_one.enabled` is false, the system shall draw as before, make no request and write nothing under `<data>/system_one/`. | Shot `565-09-off`; the fake's log and the profile's folder |
| REQ-002 | WHEN the check runs for a terminal in a listed project, the system shall POST the masked state and the `check/1` questions to the configured endpoint with the key as a bearer header and the pinned model, and show the reading in a toast and in the Decisions view. | Shot `565-02-decisions-view`; `holds` on the fake's log |
| REQ-003 | WHEN the check runs for a terminal whose project is not listed, the system shall refuse before any request and log the refusal with its reason. | Shot `565-03-refused-not-listed`; the fake's log |
| REQ-004 | WHEN the state's text holds a secret of a kind #516 hides, the system shall send and log `[redacted: <kind>]` in its place. | Shot `565-04-masked`; `holds` on the fake's log |
| REQ-005 | WHERE a project is on the metadata-only list, the system shall send code's facts alone and no text. | Shot `565-05-metadata-only`; the fake's log |
| REQ-006 | WHEN the provider does not answer within the use's deadline, answers 5xx, or is unreachable, the system shall log an `Unavailable` reading with its reason, and WHEN five calls in a row fail the system shall open the breaker for two minutes and refuse calls with that reason. | Shot `565-06-unavailable`; the log rows |
| REQ-007 | WHEN the day's spend reaches `daily_budget_cents`, the system shall refuse further calls with the budget as the reason until the next day. | Shot `565-07-budget` |
| REQ-008 | WHERE the provider is `replay`, the system shall answer from `<data>/system_one/replay.jsonl` and log the call with `replay` as its provider. | Shot `565-08-replay` |
| REQ-009 | The system shall show the key's source on the Decisions view and show or write the key's value nowhere. | Shots `565-01`, `565-02`; `grep` over the profile and Marley's log |
| REQ-010 | WHILE the Marley settings page shows, it shall show a System One section with Enabled, Provider, Endpoint, Model, Daily Budget, the check's mode and an Open Decisions link, and `default.json` shall hold the block's defaults. | Shot `565-01-settings-section`; review of `default.json` |
| REQ-011 | WHEN a setting under `marley.system_one` changes while Marley runs, the system shall apply it to the next call without a relaunch. | Shots `565-03`, `565-05`, `565-07`, `565-09` (each after a live change) |
| REQ-012 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion: `brain_ask`;
  re-verify every seam; the real answer shape. The planned live request with Chad's key could not
  run (no key in the environment, and key files are not read); his projects' recorded answers
  and working client settled the shape instead (the notes' Promotion entry).
- **P2 Code:** the ledger rows first (`Cargo.toml`: ten members and `marley_system_one` among the
  workspace dependencies; `crates/settings_content/src/marley.rs`: the `system_one` block and its
  two enums; `crates/settings_ui/src/marley_page.rs`: the section; `crates/settings_ui/src/settings_ui.rs`:
  the two dropdown renderers; `assets/settings/default.json`: the defaults); the crate; `system_one.rs`, the key, the check, `decisions.rs`, the actions; fmt and
  clippy clean (the new crate joins `script/clippy`'s cargo-shear and typos run); a review of the
  diff against each REQ, §14's adapter rule for the network and the files, and D7 (no path shows
  or writes the key).
- **P3 Test:** write and run the scenario and read every shot; `just regress`, since the Marley
  page's shot in 515 gains a section; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_system_one.md` (new) and
  `marley_workbench.md`; the plan's prong 2 section; the ledger capture; close the ticket,
  archive, commit; `brain_decide` on the provider and key decisions.
