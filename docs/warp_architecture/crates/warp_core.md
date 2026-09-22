# warp_core

> Per-crate reference (Marley round 2) — crate dir `crates/warp_core`. Marley is forked from Warp ([warpdotdev/warp](https://github.com/warpdotdev/warp)).
>
> **Provenance:** `[Warp-derived: AGPL-3.0]` — `SessionId` + `ChannelState` (server URLs / channel branding). Marley reimplements the identity as **`marley_core` `[Marley-original]`** (own `SessionId`, **no cloud/auth**); the channel/server-URL branding is a de-auth + rebrand cut, not carried. See subsystem [Provenance & licensing](../subsystems/03-terminal-session-core.md#provenance--licensing).

| Field | Value |
|-------|-------|
| Subsystem | [03 — Terminal & Session Core](../subsystems/03-terminal-session-core.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`); no own LICENSE marker. |
| Internal deps | 9 |
| Used by | **33** |

## Purpose

`warp_core` is the **app/runtime core** — the shared identity, configuration, channel/branding,
paths, feature-flag, OS-info, and telemetry layer that nearly every other crate imports. It defines
the **`SessionId`** that threads through blocks and shell DCS hooks, and the **`ChannelState`**
global that holds server URLs, app id, and channel (Stable/Preview/Oss/Dev/Local/Integration)
branding. With 33 internal dependents it is the single most load-bearing crate in this subsystem.

## Key types, modules & public API

Top-level re-exports (`src/lib.rs`): **`AppId`**, **`SessionId`**, `HostId` (from `warp_util`), and
`warpui_core`. Plus the `settings` re-export and `define_setting!` / `define_settings_group!` macros.

- **`session_id`** (`src/session_id.rs`) — `pub struct SessionId(u64)` with `as_u64()` and
  `From<u64>`/`From<SessionId> for u64`. "Each bootstrapped subshell (incl. SSH) gets its own
  `SessionId`." Defined here so low crates can reference it without depending on `app`.
- **`channel`** (`src/channel/`) — the branding/config nucleus:
  - **`Channel`** enum (`Stable`/`Preview`/`Dev`/`Local`/`Oss`/`Integration`) with
    `is_dogfood()`, `allows_server_url_overrides()`.
  - **`ChannelConfig`** (`channel/config.rs`): `server_config: WarpServerConfig`, `oz_config: OzConfig`,
    `app_id`, `logfile_name`, telemetry/crash/autoupdate/mcp configs.
  - **`WarpServerConfig`** with `production()` baking `server_root_url = "https://app.warp.dev"`,
    `rtc_server_url = "wss://rtc.app.warp.dev/graphql/v2"`, `session_sharing_server_url`.
    **`OzConfig`**, `IapConfig`, `RudderStackConfig`, `McpStaticConfig`, `McpOAuthProviderConfig`.
  - **`ChannelState`** (`channel/state.rs`) — process-global: `init()`, `new(channel, config)`,
    `set(state)`, and static accessors `server_root_url()`, `ws_server_url()`, `oz_root_url()`,
    `override_server_root_url()`, `app_id()`, `is_release_bundle()`, `channel()`, `firebase_api_key()`,
    `sentry_url()`.
- **`paths`** (`src/paths.rs`) — `warp_home_config_dir()`, `data_dir()`, `themes_dir()`,
  `warp_home_skills_dir()`, `warp_home_mcp_config_file_path()`, etc. (Warp-branded dir names.)
- **`features`** (`src/features.rs`) — runtime feature-flag menu wiring over `warp_features::FeatureFlag`
  (`DEBUG_FLAGS`, `RuntimeFeatureFlags`).
- Also: `app_id`, `platform` (`SessionPlatform`), `operating_system_info`, `telemetry`, `user_preferences`,
  `execution_mode`, `semantic_selection`, `sync_queue`, `interval_timer`, `context_flag`, `safe_log`,
  `errors`, `command`, `assertions`, `ui`, `async`, and `macos` (cfg gated).

## Depends on (internal)

- [`settings`](./settings.md) — re-exported settings framework + macros.
- [`settings_value`](./settings_value.md) — typed setting values (derive feature).
- [`string-offset`](./string-offset.md) — offset arithmetic.
- [`warp_features`](./warp_features.md) — `FeatureFlag` definitions surfaced via `features`.
- [`warp_util`](./warp_util.md) — `HostId`, path utilities.
- [`warpui_core`](./warpui_core.md) — runtime/UI primitives (re-exported for telemetry macros).
- [`warpui_extras`](./warpui_extras.md) — extra UI helpers (default feature).
- [`websocket`](./websocket.md) — WS client plumbing for server config.
- (settings/settings_value are the two `settings*` edges.)

## Used by (internal dependents)

33 crates. Notable: [`warp`](./warp.md) (the app), [`warp_terminal`](./warp_terminal.md),
[`warp_tui`](./warp_tui.md), [`ai`](./ai.md), [`mcp`](./mcp.md), [`lsp`](./lsp.md),
[`warp_cli`](./warp_cli.md), [`onboarding`](./onboarding.md), [`warp_server_auth`](./warp_server_auth.md),
[`warp_server_client`](./warp_server_client.md), [`warp_graphql`](./warp_graphql.md),
[`warp_editor`](./warp_editor.md), [`warp_completer`](./warp_completer.md),
[`node_runtime`](./node_runtime.md), [`remote_server`](./remote_server.md),
[`integration`](./integration.md), [`vim`](./vim.md). Plus cloud_object_*, http_client/http_server,
repo_metadata, simple_logger, ui_components, warp_files, warp_logging, warp_managed_secrets,
warp_multi_agent_client, warp_search_core, warp_isolation_platform, warp_channel_config.

## Related crates

- [`warp_terminal`](./warp_terminal.md) — pairs with `SessionId` for the block model.
- [`warp_server_auth`](./warp_server_auth.md) / [`warp_server_client`](./warp_server_client.md) — the
  auth/network layer driven by `ChannelState` server URLs (de-auth target).
- [`warp_features`](./warp_features.md), [`settings`](./settings.md) — config siblings.

## Marley relevance

**Classify: KEEP + EXTEND, RENAME-deferred.** This crate is the **primary touchpoint for goals
(3) de-auth + login stub and (4) de-Warp rebrand**:
- **De-auth / login stub:** `ChannelState::server_root_url()` / `ws_server_url()` / `oz_root_url()`
  and `OzConfig`/`IapConfig` are what point the app at Warp's auth + GraphQL backend. For an offline
  boot, **stub these** to a local/no-op endpoint (e.g. construct `ChannelState::new(Channel::Oss,
  …)` with a `WarpServerConfig` pointing at a Marley stub, or short-circuit the accessors) so login
  isn't required. Note `Channel::Oss` already `allows_server_url_overrides() == false`, so to redirect
  shipped builds you must change the baked `production()` URLs, not rely on env overrides.
- **Rebrand:** `WarpServerConfig::production()` URLs, `paths::warp_home_config_dir()` (the `~/.warp`
  dir name), `logfile_name`, and `AppId::new("dev","warp",…)` carry Warp branding to change to Marley.
- **Rename:** ideally `warp_core` → `marley_core`, but with **33 dependents** this is the highest-cost
  rename in the batch — **defer** to a late mechanical pass, or alias.
Don't touch `SessionId` semantics — goal (2) spawn/write/read depends on it being stable.

## Notes / gotchas

- `ChannelState` is a **process-global singleton** (`set`/static accessors). Tests use
  `mock_server()` (mockito) under `test-util`; forgetting `ChannelState::set(...)` panics accessors.
- `build.rs` present (small). macOS pulls `objc2-foundation`; non-wasm pulls `tokio`.
- Feature flags: `crash_reporting` (sentry), `integration_tests`, `local_fs`, `release_bundle`,
  `test-util`. `release_bundle` changes path resolution — relevant when packaging Marley.
- Paths use Warp-named dirs; changing them affects where settings/skills/MCP config are read.
