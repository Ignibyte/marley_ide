# Forge as a fleet provider — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-841-forge-as-a-fleet-provider.md
- **Pipeline spec:** 841-forge-provider.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after W3.4 (#836–#840, #868–#872).
- **Classification / tier:** feature, L. Marley crates and `settings_content` (Marley's own
  section of it); no Zed hunk.
- **Recall (§18.3):** the fleet contract (#606 era) and its remotes in `fleet_providers.rs`;
  System One's keyring (#615 era). The brain: nothing on this seam.
- **Discovery (Explore):** Forge serves rmcp Streamable HTTP, stateless, at `/mcp/forge` (default
  `127.0.0.1:8080`), bearer `forge_pk_…`; `agent-list {project_id?}` returns `{agents:
  [AgentSessionDto]}` in `structuredContent` (id, project_id, agent_name, folder, host, harness,
  status running|stopped, current_ticket?, last_output_at?, last_heartbeat_at, state
  working|needs_attention|idle). Marley's providers are `FleetProviderContent` kinds polled by
  `fleet_providers::poll_once` (`work_handshake`, `work_agents`, optional `work_agent`); the Fleet
  panel groups by `AgentSummary.host_id`, naming an undescribed host by its id.
- **Decisions:** D1–D4 in the spec.

### Design
- **`settings_content/src/marley.rs`:** `FleetProviderContent::Forge { name: Option<String>, url:
  String, projects: Option<Vec<String>>, poll_seconds: Option<u64>, bearer_env: Option<String> }`
  (kind `"forge"`); the settings JSON schema follows; `default.json`'s comment names it.
- **`fleet_providers.rs`:** the remote for `Forge` connects as `Mcp { url }` does, the bearer from
  `cx.read_credentials(url)` (the password bytes) else `bearer_env`; `poll_once` for a Forge remote
  makes the handshake itself (`marley.work/v1`, provider "forge", `poll_s` from `poll_seconds`,
  default 15) and calls `agent-list` once per project id (or once with none), merging; `forge.rs`
  (new, beside it) holds the `AgentSessionDto` subset (serde, unknown fields ignored) and
  `summary(dto, now)`: id, `agent_name`, `harness` as runtime, D2's state, `last_output_at` as
  `last_seen_ms`, `host` as `host_id`, `current_ticket` as the work item's title, `folder` in the
  work item's detail. `work_agent` is not offered (no capability), so the row has no detail pane.
- **`marley: set forge token`:** a small `ModalView` (`forge.rs`) with a masked single-line editor
  for the first forge provider's URL; Enter writes `cx.write_credentials(url, "forge", token)`,
  Escape dismisses; a toast says whether it was written.
- **File manifest:** `crates/settings_content/src/marley.rs`, `assets/settings/default.json`,
  `crates/marley_workbench/src/fleet_providers.rs`, `forge.rs` (new), `marley_workbench.rs`
  (module, action, init); the scenario. `settings_content` is Zed's crate but `marley.rs` in it is
  Marley's own file (its ledger row covers it).
- **Visual check plan:** REQ-001/002 by the Fleet panel against a stand-in Forge; REQ-003 by the
  modal shown and dismissed (writing would touch the user's keyring); REQ-004 by the stand-in's
  request log, empty before the provider is set. Not reached: the keyring read (the scenario uses
  `bearer_env`), several projects.
- **Risks:** `ContextServer::http`'s session handling against a stateless server (it must accept a
  missing `Mcp-Session-Id`); the stand-in shows whether it does.

## Phase 2 — Code
- **Built:** `FleetProviderContent::Forge` (kind `forge`) in `settings_content/src/marley.rs`, its
  line in `default.json`'s fleet comment; in `fleet_providers.rs`, `name_of` ("Forge" or the
  entry's name), `connect` for Forge (`ContextServer::http` with the bearer from
  `keyring_token(url)`, else `bearer_env`), and in `poll_once` a Forge branch that makes the
  handshake (`forge_handshake`: the contract, provider "forge", no capabilities, `poll_s` from
  `poll_seconds` or 15, `stale_after_s` 300) and calls `poll_forge`: `agent-list` once per project
  id or once with none. `forge.rs`: `ForgeAgents`/`ForgeAgent` (the fields read), `summaries`
  (stopped sessions left out), `summary` (D2's states; the work item's key is the ticket, the
  folder and, for a waiting session, "silent N m"; `last_seen_ms` the heartbeat,
  `state_since_ms` the last output), and `marley: set forge token` (`SetForgeToken`, the
  `ForgeToken` modal: a masked editor, Enter writes `cx.write_credentials(url, "forge", token)`
  and toasts the result, Escape dismisses; a toast when no Forge provider is set).
- **Deviations:**
  - The first run's rows read "stale": `last_seen_ms` was the last output, minutes old. It is
    the heartbeat now, and the handshake's `stale_after_s` is 300, since Forge leaves the
    heartbeat's cadence to each harness.
  - The row's second line carries the folder and the silence through the work item's key: the
    Fleet row draws only the key, and D2 wants the silence on the row.
- **Gate:** red once on clippy (`poll_once` past 100 lines, so `client_of` starts the client for both branches; `option_if_let_else` on the token, now `or_else`; `unused_self` on the modal's cancel, now a closure; `needless_pass_by_ref_mut` on `init`), fixed at source. The scenario ran again on the fixed code, with the same shots and log.
- **Review:** the token is never logged or shown (the modal's field is masked; the keyring read
  logs only through `log_err` on a failure, with no value); one Forge provider's failure backs
  off on its own as the other remotes do; no Zed code changed beyond Marley's own settings file.


## Phase 3 — Test
- **Scenario:** `script/e2e/841-forge-provider.sh` (sway): a stand-in Forge in Python serving
  stateless JSON-RPC at `/mcp/forge`, refusing a wrong bearer, logging each request; five
  sessions, one stopped; the token through `bearer_env` (`FORGE_E2E_TOKEN`).
- REQ-004: with the Fleet panel open and no forge provider, the stand-in's log had 0 requests.
  With the provider set: `initialize`, `notifications/initialized` and 8 `tools/call`, every one
  with the bearer.
- `841-01-fleet`: the Fleet tab, "Forge ready", hosts forge, agent1, agent2, agent3 each with its
  agent: code-1 `#826 · /srv/stacks/marley_ide` working; code-2 `#828 · /srv/work/marley_ide ·
  silent 7 m` waiting; qa-1 (Codex's icon) `#829 · …` working; docs-1 `/srv/work/rusty-v3`
  idle; the stopped session absent. REQ-001, REQ-002 met.
- `841-02-token`: "Forge's token", "Kept in the system keyring for http://127.0.0.1:…/mcp/forge.",
  the typed token masked; Escape before Enter. REQ-003's field met; the write is not run, since it
  would touch the user's keyring.
- `841-03-day`: Marley Day, the same list. REQ-001 met.
- Not reached: the keyring read and write, several projects, a Forge that refuses the token
  (the provider's header would read unreachable, as for any remote).


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (Forge as a fleet provider), `fleet-contract.md`
  (Forge's adapter), look-plan W4.1 done, the ledger row for `marley.rs`.
- Knowledge: L-claude-841-a-stand-in-mcp-server-over-http-001. Brain: `brain decide` on
  consultation 347107b2.
- Ticket closed; pipeline archived.
