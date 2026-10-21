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

## Phase 3 — Test

## Phase 4 — Complete
