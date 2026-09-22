# warp_server_client

> Per-crate reference (Marley round 2). Crate dir: `crates/warp_server_client`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** the authenticated transport to `app.warp.dev` (token engine, device flow, IAP). Marley is local-first → **N/A today**; future-brain *pattern* only, re-invent from permissive deps. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (no own `LICENSE` marker; `license.workspace = true`) |
| Internal deps | 13 |
| Used by | 2 |

## Purpose

`warp_server_client` is the **action layer** over auth state: it performs the actual network login, token refresh, and authenticated requests against the Warp server. It owns the HTTP base client, the OAuth2/Firebase credential-exchange flow, device-code (headless CLI) auth, IAP (Identity-Aware Proxy) token handling, API-key management, user-settings sync, and Warp Drive access. Where [`warp_server_auth`](./warp_server_auth.md) *models* who is logged in, this crate *makes it happen* and keeps tokens fresh. It is the broadest crate in the subsystem (13 internal deps).

Feature flags: `skip_login` (forwards to `warp_server_auth/skip_login`), `integration_tests`, `agent_mode_evals`, `test-util`.

## Key types, modules & public API

`src/lib.rs` declares `pub mod auth, base_client, drive, graphql_helpers, iap, ids, network_logging` (+ private `public_api`); re-exports `auth::UserUid`, `cloud_objects::server_id_traits`, and `public_api::HttpStatusError`.

- **`base_client`** (`src/base_client.rs`):
  - `pub struct BaseClient` — `new(...)`, `http_client()`, `owned_http_client() -> Arc<Client>`, `auth_session() -> Arc<AuthSession>`, `anonymous_id()`, `user_id() -> Option<UserUid>`, `get_or_refresh_access_token() -> Result<AuthToken>`, `event_sender()/send_auth_event()`, `graphql_request_options()` / `graphql_request_options_with_token()` (builds `warp_graphql::RequestOptions`), `ambient_headers()`, `observe_iap_challenge()`, `wrap_eventsource_with_iap_detection()`, `report_ws_iap_challenge()`, `iap_proxy_auth_header()`.
  - `pub struct AmbientHeaderPolicy` (`inherit_all` / `for_task(id)` / `workload_only` / `omit_all`), `pub enum HeaderOverride<T>`, `pub struct GraphqlRoutingConfig`, `pub struct AuthenticatedGraphqlConfig`.
- **`auth`** (`src/auth/mod.rs`, `src/auth/session.rs`):
  - `pub trait AuthClient: Send + Sync` — the full server-auth surface: `create_anonymous_user`, `get_or_refresh_access_token`, `fetch_user`, `fetch_new_custom_token` / `on_custom_token_fetched`, `fetch_user_properties`, `get_user_settings` / `update_user_settings` (+ `set_is_telemetry_enabled`, `set_is_crash_reporting_enabled`, `set_is_cloud_conversation_storage_enabled`, `set_user_is_onboarded`), `request_device_code` / `exchange_device_access_token` (OAuth2 device flow), `list_api_keys` / `create_api_key` / `expire_api_key`, `list_agent_identities`.
  - `pub struct AuthClientImpl` — the concrete impl over `Arc<BaseClient>`.
  - `pub struct AuthSession` — combines auth state with HTTP transport; `get_or_refresh_access_token()`, `exchange_credentials()`, `request_device_code()`, `exchange_device_access_token()`; emits `pub enum AuthEvent { NeedsReauth, AccessTokenRefreshed { .. }, .. }` over an `async_channel`.
  - Result/error types: `FetchUserResult`, `SyncedUserSettings`, `AgentIdentity`, `pub enum UserAuthenticationError`, `pub enum MintCustomTokenError`. Re-exports `warp_server_auth::user_uid` + `TEST_USER_EMAIL/TEST_USER_UID/UserUid`.
- **`iap`** (`src/iap.rs`) — Google IAP proxy support: `pub struct IapState` (`new(config)`, `get_cached`, `proxy_auth_header`, `state`), `pub enum IapCredentialsState`, `pub struct CachedToken`, `pub type PathResolver`; reads `WARP_IAP_TOKEN`, proactive refresh, retry/backoff (singleton entity).
- **`drive`** (`src/drive.rs`) — `pub use cloud_objects::drive::*` (Warp Drive surface re-exported).
- **`public_api`** — `pub struct HttpStatusError`. **`graphql_helpers`**, **`network_logging`**, **`ids`** — request helpers, logging, id types.

## Depends on (internal)

- [`warp_server_auth`](./warp_server_auth.md) — the auth-state model this crate populates.
- [`warp_graphql`](./warp_graphql.md) — typed operations + `RequestOptions` for every server call.
- [`firebase`](./firebase.md) — Google/Firebase REST response decoding during credential exchange.
- [`http_client`](./http_client.md) — the underlying `reqwest` client.
- [`websocket`](./websocket.md) — WS transport (non-wasm) incl. IAP error mapping.
- [`cloud_objects`](./cloud_objects.md), [`cloud_object_client`](./cloud_object_client.md), [`cloud_object_models`](./cloud_object_models.md) — Warp Drive objects.
- [`command`](./command.md) — command/run context used by clients.
- [`settings_value`](./settings_value.md) — typed settings for user-settings sync.
- [`warp_isolation_platform`](./warp_isolation_platform.md) — platform/isolation hooks.
- [`warp_core`](./warp_core.md) — `IapConfig`, channel/runtime primitives.
- [`warpui_core`](./warpui_core.md) — `AppContext`/entity/async (`Timer`, `BoxFuture`) integration. **(MIT)**

## Used by (internal dependents)

- [`warp`](./warp.md) — the top-level app crate.
- [`warp_multi_agent_client`](./warp_multi_agent_client.md) — multi-agent orchestration that authenticates through this client.

(Total internal dependents: 2.)

## Related crates

- [`warp_server_auth`](./warp_server_auth.md) — paired state crate; read together.
- [`warp_graphql`](./warp_graphql.md) — the operation library this drives.
- [`firebase`](./firebase.md) — credential wire types.
- [`http_client`](./http_client.md) / [`websocket`](./websocket.md) — transports.

## Marley relevance

**Classification: KEEP + STUB (primary network short-circuit point).**

Together with `warp_server_auth`, this is where Marley's **(3) de-auth + login stub** is implemented in behavior (auth *actions* live here):

- **Stub the `AuthClient` impl.** Provide a Marley `AuthClient` (or feature-gate `AuthClientImpl`) whose `get_or_refresh_access_token` returns a static test token, whose `fetch_user` yields a canned `FetchUserResult`, and whose `create_anonymous_user` succeeds offline — so nothing ever calls Firebase/Warp. The `mockall`-based `test-util` mock of `AuthClient` is a ready-made template. Combine with `skip_login` to suppress the login UI entirely.
- **(2) session spawn/write/read:** `BaseClient` is consulted for ids/headers when sessions sync to the cloud; with a stub client these calls succeed locally and never block session spawn. Audit `drive`/cloud-object writes so local sessions don't require a live server.
- **(1) custom panel:** A Marley status/account panel can read `BaseClient::user_id()` / `AuthState` to show "offline / local" cleanly.
- **(4) de-Warp rebrand:** Env vars `WARP_IAP_TOKEN` (iap.rs) and `WARP_USER_SECRET` (via auth crate), plus `dev@warp.dev`/`test_user@warp.dev` strings, are rebrand targets. IAP and device-code flows are Warp-cloud-specific and can be **STUBBED to no-ops** for offline builds. Package rename `warp_server_client → marley_server_client` has 2 dependents → cheap.

Do not REMOVE: `warp` and `warp_multi_agent_client` depend on its types and the `AuthClient` trait. Keep the surface, neuter the network.

## Notes / gotchas

- **Largest dep fan-in here (13 internal crates).** Changes ripple; prefer stubbing impls over editing the trait surface.
- **IAP is GCP-specific.** `iap.rs` shells out to credential resolution (e.g. gcloud) and caps retries at `MAX_FAILURE_RETRIES = 5`; entirely dead weight off Warp's infra — safe to stub to `IapCredentialsState`-none.
- **OAuth2 device flow** (`request_device_code` / `exchange_device_access_token`) uses the external `oauth2` crate's `StandardDeviceAuthorizationResponse` — this is the headless CLI/SDK login path.
- **wasm gating:** `diesel`(sqlite) and `websocket` deps and several IAP/WS methods are `cfg(not(target_family = "wasm"))`.
- **`drive` is just a re-export** of `cloud_objects::drive::*` — the real Warp Drive types live in `cloud_objects`.
- `edition = "2024"`. `schemars` is pinned to `"1"` directly (not workspace) in `Cargo.toml`.
