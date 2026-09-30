# The marley.work/v1 clients over MCP and HTTP — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-611-work-provider-clients.md
- **Pipeline spec:** 611-work-provider-clients.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - #534's spec plans the same ContextServer::stdio client for the harness.
  - No Marley crate depends on context_server yet; Zed's agent calls tools at context_server_registry.rs:362-395 but reads content, not structured_content.
  - system_one.rs is the http_client precedent (bearer header, timeout, backstop timer).
  - marley_mcp's rule: a bearer never enters a log, a prompt or a file other than the user's own config.
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:**
  - #610: the loop that reads the fleet runs slow work between reads, outside any update; the
    providers' calls join it the same way.
  - PR-claude-607: reads run only while a Fleet surface shows.
  - marley_mcp's bearer rule: a token never enters a log, a prompt or a file.
  - The brain (consultation 08a59e4270914224b603e72e71a50edf): nothing on this seam.
- **Seams re-verified:**
  - `ContextServer::stdio`, `::http`, `start(&AsyncApp)`, `client()`, `stop()`
    (`context_server.rs:47-170`); `InitializedContextServerProtocol::request::<T>`
    (`protocol.rs:107`); `CallToolParams { name, arguments, meta }` and `CallToolResponse {
    content, is_error, structured_content }` with `text_contents()` (`types.rs:164, 719-731`).
  - `ContextServerCommand { path, args, env, timeout }` (`settings_content/src/project.rs:521`).
  - `context_server` is a workspace crate (`Cargo.toml:332`) that `marley_workbench` does not yet
    depend on; `http_client` it does.
  - `system_one::post` (`system_one.rs:890-945`): `Request::builder()`, `.header("Authorization",
    …)`, `.timeout(…)`, `client.send`, `read_to_string`, a backstop timer.
  - `marley_sdk::Changes { cursor, reset, changed: [Changed { kind, id }] }`.

### Design
- **Approach.**
  - *Settings* (Zed crate, additive): `FleetProviderContent` gains `Mcp { name, command, args,
    url, bearer_env }` and `Http { name, url, bearer_env }`; its touchpoint row first.
  - *A new file, `crates/marley_workbench/src/fleet_providers.rs`*:
    - `Remote`, one per `mcp` or `http` entry, keyed by the entry itself: its client
      (`Client::Mcp(Arc<ContextServer>)` or `Client::Http { base, bearer }`), its handshake, its
      `Status` (`Connecting`, `Ready`, `Unreachable(reason)`, `Incompatible(contract)`), its list,
      the details of wanted agents, its cursor, when it last answered, its failure count and when
      it may be tried again. `Debug` by hand, without the bearer.
    - `sync(remotes, entries)`: keeps a `Remote` whose entry is still set, starts one for a new
      entry, and stops an MCP server whose entry went.
    - `poll_all(remotes, wanted, cx)`: each remote due (its `poll_s`, or its backoff) polls
      concurrently (`join_all`): start its client if none (MCP: `ContextServer::stdio` or
      `::http` with the bearer header, then `start`; HTTP: the bearer read from `bearer_env`), the
      handshake if none (a contract other than `marley.work/v1` is `Incompatible`), then the list
      (`work_agents`, or `work_changes` with the cursor when `changes` is offered and the list
      only on a reset or a changed agent) and `work_agent` for each wanted agent it lists. Each
      call gives up after 5 s: a timeout changes nothing, so the source reads stale after three
      polls with no answer; an error is `Unreachable` with its first line, a backoff, and for MCP
      the server stopped so the retry starts it again.
    - `call`: over MCP, `CallTool` and `structured_content`, else the JSON of the text content,
      `is_error` an error; over HTTP, `GET <base>/marley/v1/…` with the bearer, a non-2xx status
      an error ("HTTP 401").
    - `source(remote, now)`: the remote as a `Source` whose `state` says what the header shows.
    - The bearer is read once when the client starts, logged only as "reads its bearer from
      $NAME", and never put in an error.
  - *`fleet.rs`*: `Source.failure` becomes `Source.state` (`Ready`, `Failed`, `Connecting`,
    `Unreachable`, `Stale`, `Incompatible`), drawn as a chip in the source's header with the
    reason beside it; `agent_chip` reads `offline` for a source that is unreachable; `Fleet.remotes`;
    the loop polls the remotes after the hosts and before the read; `read_providers` gives each
    `mcp` or `http` entry its remote's source, in the settings' order.
  - *The stub*, `script/e2e/fleet-stub-provider.py`: Python, no dependencies; `http <port>` or
    `mcp` on stdio; the fixtures moved to now; a control file read at each request; an optional
    token checked on HTTP.
- **File manifest:**
  - Marley: `crates/marley_workbench/src/fleet_providers.rs` (new), `fleet.rs`,
    `marley_workbench.rs`, `crates/marley_workbench/Cargo.toml` (`context_server`),
    `script/e2e/fleet-stub-provider.py` (new).
  - Zed: `crates/settings_content/src/marley.rs` (row first).
- **Visual check plan** (`script/e2e/611-work-provider-clients.sh`, sway; the HTTP stub on a
  port of its own with a token, the MCP stub over stdio; `FLEET_TOKEN` exported for Marley):

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001 | Both providers set, the panel open | `ready.png`: both headers ready, both lists |
  | REQ-002, REQ-006 | Kill the HTTP stub | `unreachable.png`: its header unreachable with the reason, its agents offline, the MCP provider's list as it was |
  | REQ-003 | Start the HTTP stub again answering `v9` | `incompatible.png`: its header naming `marley.work/v9` |
  | REQ-004 | Control `ok`, then `silent` | `stale.png`: its header stale, its last list kept |
  | REQ-005 | Grep Marley's log for the token and for `FLEET_TOKEN` | `log.txt` |

- **Risks and decisions:**
  - A silent provider holds only its own poll: the remotes poll concurrently, each call bounded
    by 5 s.
  - `work_run` is left for when a tab wants more than the detail's run.

## Phase 2 — Code (2026-09-30)
- **Built,** as designed:
  - Settings: `FleetProviderContent::Mcp { name, command, args, url, bearer_env }` and
    `Http { name, url, bearer_env }` (`settings_content`, its row first).
  - `fleet_providers.rs`: `Remote` (with a `Debug` that leaves the client, and so the bearer,
    out), `Client::{Mcp(Arc<ContextServer>), Http { base, bearer, http }}`, `Status`, `Failure`
    (`TimedOut`, `Incompatible`, `Failed`); `name_of`; `sync`; `poll_all` (`join_all`), `poll`
    (ready, a timeout that changes nothing, incompatible, unreachable with its first line and
    the MCP server stopped), `back_off` (1, 2, 4, 8, 16, then 30 s), `poll_once` (client,
    handshake checked against `WORK_CONTRACT`, `work_changes` when `changes` is offered, the list
    on a reset or a changed agent, `work_agent` for wanted agents), `connect`
    (`ContextServer::stdio` with `ContextServerCommand`, `ContextServer::http` with the bearer
    header, or the plain HTTP client), `web_url`, `bearer` (logs the variable's name only),
    `encode`, `call` (bounded by `CALL_TIMEOUT` with a timer in `select`), `mcp_value`
    (`structured_content`, else the JSON of the text, `is_error` an error), `get`, `source`.
  - `fleet.rs`: `SourceState` (`Ready`, `Failed`, `Connecting`, `Unreachable`, `Stale`,
    `Incompatible`) in place of `failure`, `Source::for_state`, a state chip and its reason on
    each store's header (`source_chip`, `source_reason`), `offline` agents in an unreachable
    source, `Fleet.remotes`, the loop polling the remotes after the hosts, `keep_remotes`,
    `wanted`, and each `mcp` or `http` entry's source in `read_providers`.
  - `Cargo.toml`: `context_server` for the workbench.
  - `script/e2e/fleet-stub-provider.py`: HTTP and MCP stdio, the fixtures moved to now, a
    control file, a token on HTTP, `changes` over MCP. Tried by hand: 401 without the token,
    the handshake, the list and a detail with its host with it; MCP's `initialize` and
    `tools/call` with `structuredContent`.
- **Deviations:** `work_run` is not called (the spec's scope now says so). `if let … else`
  and `map_or_else` where clippy's `single_match_else` and `option_if_let_else` asked.
- **Review of the diff:**
  - REQ-001: a ready handshake and list make a ready source with its agents.
  - REQ-002: an error is `Unreachable` with its first line; the source's agents read `offline`;
    the backoff grows to 30 s; the MCP server is restarted on the retry.
  - REQ-003: a handshake with another contract is `Incompatible`, named, retried with backoff.
  - REQ-004: a call that times out changes nothing, so three polls without an answer read stale.
  - REQ-005: the token is read from the named variable when the client starts, sent only in
    the `Authorization` header, and never formatted into a log line or an error; `Remote`'s
    `Debug` leaves the client out.
  - REQ-006: each remote polls on its own and fails on its own; `join_all` runs them together.
  - Re-entrancy: the calls await between `cx.update` calls; `sync` and `keep_remotes` touch only
    the global.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/611-work-provider-clients.sh` under sway: the stub over HTTP on a
  port of its own with the token `STUB_TOKEN`, the stub over MCP started by Marley, and
  `FLEET_TOKEN` exported for Marley; the providers `store over HTTP` (with `bearer_env`) and
  `store over MCP`.
- **Shots, each read** (run 2; the panel cropped from each and read side by side):
  - `ready.png` (REQ-001): "store over HTTP" `ready` and "store over MCP" `ready`, each with
    build-1 `working`, review-1 `waiting` with its warning mark and docs-1 `error`, under their
    hosts (named by id, `host-build-1` and `host-vps-2`, since no detail with the host's name
    was wanted).
  - `unreachable.png` (REQ-002, REQ-006): the HTTP store killed: its header `unreachable` with
    "Connection refused (os err…" (the full chain in the tooltip, and logged once), its three
    agents `offline`; the MCP store `ready` with its list as it was.
  - `incompatible.png` (REQ-003): the HTTP store back, answering `marley.work/v9`: its header
    `incompatible` "speaks marley.work/v9", no agents; the MCP store untouched.
  - `answering.png`: the HTTP store answering again after its backoff: `ready`, its list back.
  - `stale.png` (REQ-004): the HTTP store silent: its header `stale` "no answer for three
    polls", its last list kept with every agent `stale`; the MCP store `ready`.
  - `log.txt` (REQ-005): only lines "fleet: provider "store over HTTP" reads its bearer from
    $FLEET_TOKEN", and "the token itself is not in the log".
  - `work_changes` ran for the MCP store (it offers `changes`): its list stayed whole through
    every shot.
- **Red found and fixed** (run 1): the unreachable header's reason was the outer error ("error
  sending request for u…"), cut by the header's width before the cause. The header now shows the
  root cause, the reason has a tooltip, and the whole chain is logged once when a store becomes
  unreachable. `just gate-diff` after the fix: 17 PASS, 0 FAIL; run 2 green.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after; the HTTP stub was killed in `teardown`.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Added: real workflow stores in the Fleet panel);
  `three-prong-plan.md` (#611 shipped, wave 1 done, the brain's own implementation next);
  `fleet-contract.md` (the providers' settings, `changes`, `work_run` not yet called);
  `marley_workbench.md` (the stores' states in the Fleet section, and a section for
  `fleet_providers.rs`); the guide (a Real stores section) and the guide page (article
  `fleet-stores` and its nav entry). The walkthrough gains nothing: a check needs a store that
  serves the contract, which the dev box does not run yet. The `settings_content/src/marley.rs`
  row was written before the code and describes what shipped.
- **Knowledge:** F-claude-611-an-unreachable-stores-header-showed-its-outer-error-001,
  AD-claude-611-workflow-stores-are-polled-together-each-call-bounded-001.
- **Brain:** consultation 08a59e4270914224b603e72e71a50edf closed with `brain decide`
  (`decisions/marley-polls-its-workflow-stores-together-over-mcp-or-http-each-call-bounded`,
  follow-up by 2026-10-14).

