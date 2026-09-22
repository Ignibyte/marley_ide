# Subsystem 05 — Cloud, Auth & Networking Seam

> Part of the Marley architecture docs. Marley is forked from Warp (warpdotdev/warp),
> which is **AGPL-3.0**. Round 3 re-review (2026-07-12): verified against the current
> workspace + refreshed. **See [Marley status @ M15](#marley-status--m15) and
> [Provenance & licensing](#provenance--licensing) first** — they reframe the whole
> subsystem. Short version: Marley is **local-first with zero networking
> dependencies**, so *none* of the 18 crates below were adopted, and the round-1
> "de-auth dossier" framing (how to *stub* Warp's mandatory login) is now largely
> moot — there is no Warp auth boundary in Marley to neutralize. The §1–§13 map is
> retained as an accurate description of **Warp's** upstream design and as the
> reference for a *future* brain/cloud offering.

This document maps the boundary between the Warp client and Warp's cloud
(`app.warp.dev` and friends): how credentials are represented, how the client
authenticates, which endpoints it talks to, where login is *enforced*, and what
keeps working when you are anonymous or fully offline. It is the upstream input
for the separate **de-auth dossier**, so the "where login is gated" sections are
deliberately exhaustive and cite exact files/lines.

---

## 0. The 30-second mental model

```
                 baked-in URLs (warp_core::channel::config)
                 ┌───────────────────────────────────────────┐
   Credentials   │ app.warp.dev      ── GraphQL /graphql/v2   │
   (warp_server_ │ rtc.app.warp.dev  ── Warp Drive WS subs    │
    auth)        │ sessions.app...   ── session sharing WS    │
       │         │ oz.warp.dev       ── ambient/cloud agents  │
       ▼         │ securetoken/identitytoolkit.googleapis.com │
  AuthState ◄──► │ (Firebase auth, hardcoded API key)         │
       │         └───────────────────────────────────────────┘
       ▼                          ▲
  AuthSession  ──► http_client (reqwest + X-Warp headers + IAP)
  (refresh /        websocket   (native/wasm graphql-transport-ws)
   device flow)
       │
       ▼
  ServerApi / AuthClient / cloud_object_client  ──► typed GraphQL (cynic)
       │
       ▼
  app/src/auth/auth_manager.rs  ──► is_anonymous_or_logged_out()  ◄── ~30 UI gates
```

Everything below the Credentials line is the **transport/seam** (the crates in
this subsystem). The *policy* of "you must be logged in to do X" lives one layer
up in the `app` crate (`app/src/auth/`), and is summarised in §7.

---

## 1. Server topology & where the URLs are baked in

All cloud endpoints are compile-time constants, not config files. They live in
**`crates/warp_core/src/channel/config.rs`**:

| Endpoint | Const | Used for |
|---|---|---|
| `https://app.warp.dev` | `WarpServerConfig::server_root_url` (`config.rs:59`) | GraphQL (`/graphql/v2`), REST (`/api/v1/...`), Firebase token proxy (`/proxy/token`, `/proxy/customToken`), OAuth device flow (`/api/v1/oauth/...`) |
| `wss://rtc.app.warp.dev/graphql/v2` | `rtc_server_url` (`config.rs:60`) | Warp Drive real-time subscriptions (`get_warp_drive_updates`) |
| `wss://sessions.app.warp.dev` | `session_sharing_server_url` (`config.rs:61`) | Terminal session sharing |
| `https://oz.warp.dev` | `OzConfig::oz_root_url` (`config.rs:82`) | Ambient / cloud agent ("Oz") dashboard + workload identity audience |
| `AIzaSy…<Warp public Firebase web key, redacted>` | `firebase_auth_api_key` (`config.rs:62`) | Google Firebase Auth REST API key (hardcoded, shipped in binary) |
| `https://securetoken.googleapis.com/v1/token` | `FirebaseToken::access_token_url` (`credentials.rs`) | Refresh-token → access-token exchange |
| `https://identitytoolkit.googleapis.com/v1/accounts:signInWithCustomToken` | same | Custom-token (anonymous) → access-token exchange |

**Channel selection.** `ChannelState::init()`
(`warp_core/src/channel/state.rs:37`) hard-defaults the open-source build to
`Channel::Oss` with `WarpServerConfig::production()`. So the **default Marley
build already points at `app.warp.dev`.** The channel enum is in
`warp_core/src/channel/mod.rs:10` (`Stable`, `Preview`, `Dev`, `Local`, `Oss`,
`Integration`).

**URL overrides are blocked for OSS.** `Channel::allows_server_url_overrides()`
(`mod.rs:42`) returns `false` for `Stable`/`Preview`/`Oss` and `true` only for
`Dev`/`Local`/`Integration`. `app/src/lib.rs:650` only applies the
`--server-root-url` / `WARP_SERVER_ROOT_URL` (`warp_cli/src/lib.rs:44,125`)
override when `allows_server_url_overrides()` is true. **Net effect: an OSS-channel
binary cannot be redirected away from `app.warp.dev` via flags** — you must change
the channel, change the baked config, or call
`ChannelState::override_server_root_url()` (`state.rs:90`) yourself. This is a
primary Marley de-auth lever.

---

## 2. `warp_server_auth` — credential model & in-memory auth state

The source of truth for "who is logged in." `crates/warp_server_auth/src/`:

- **`credentials.rs` — `Credentials` enum** (the central type):
  - `Firebase(FirebaseAuthTokens)` — id token (short-lived) + refresh token (long-lived). The normal logged-in path.
  - `ApiKey { key, owner_type }` — direct server auth; keys are prefixed `wk-` (`API_KEY_PREFIX`, `lib.rs:11`). Used by CLI/SDK/service accounts.
  - `Bearer(String)` — externally-managed token handed in by the **remote-server daemon** handshake (§9). `is_externally_managed()` → skips local refresh/reauth.
  - `SessionCookie` — ambient browser cookie auth for **Warp on Web** (no Authorization header; `AuthToken::NoAuth`).
  - `Test` — only compiled under `test` / `integration_tests` / `skip_login`.
  - Helper types: `AuthToken` (what actually goes in the `Authorization: Bearer` header), `LoginToken` (long-lived half), `FirebaseToken::{Refresh,Custom}` with `access_token_url`/`proxy_url`/`access_token_request_body` describing the Firebase REST exchange.

- **`auth_state.rs` — `AuthState`**: holds `Option<User>`, `Option<Credentials>`, an `anonymous_id` UUID, and a `needs_reauth` flag, all behind `RwLock`/atomics. Exposed app-wide as the `AuthStateProvider` singleton.
  - **`AuthState::initialize(ctx, api_key)`** (`auth_state.rs:~117`) is the boot-time credential resolver, in priority order: (1) **test user** if `should_use_test_user()`, (2) provided **API key**, (3) **`WARP_USER_SECRET`** compile-time env var (`option_env!`) deserialized into a `PersistedUser`, (4) **persisted user** from OS secure storage (`PersistedUser::from_secure_storage`).
  - **`should_use_test_user()`** (`auth_state.rs:~178`) = `cfg!(any(test, feature="skip_login", feature="test-util"))` **OR** channel == `Integration`. When true it injects `User::test()` + test credentials and returns immediately — i.e. a built-in "logged-in without a server" mode.
  - Key predicates consumed by the rest of the app: `is_logged_in()` (= credentials present), `is_anonymous_or_logged_out()` (the master UI gate, §7), `is_user_anonymous()`, `needs_reauth()`, `is_api_key_authenticated()`, `is_service_account()`.
  - `persist_action()` decides whether to write/remove the user in secure storage; only `Firebase` credentials are persisted (API key / Bearer / SessionCookie / Test → `DoNothing`).
  - Implements `warp_managed_secrets::ActorProvider` (bridges the logged-in UID into secret encryption, §10).

- **`user.rs` / `user/persistence.rs`** — `User`, `UserMetadata`, `FirebaseAuthTokens`, `AnonymousUserType` (`WebClientAnonymousUser`, `NativeClientAnonymousUserFeatureGated`, …), `PrincipalType` (`User`/`ServiceAccount`), `PersonalObjectLimits`, and the `PersistedUser` secure-storage record.
- **`anonymous_id.rs`** — stable random UUID for un-logged-in telemetry/identity.
- **`skip_login` feature** is defined here (`Cargo.toml:11`) and propagated to `warp_server_client` and `app`.

---

## 3. `firebase` — Google Identity REST DTOs

`crates/firebase/src/lib.rs` is a thin, dependency-light crate of serde DTOs for
the Firebase/Identity-Platform REST API:
- `FirebaseError` (Google standard `{code,message}`), `AccountInfo`, `GetAccountInfoResponse`, `FetchAccessTokenResponse` (handles both camelCase and snake_case because the refresh-token and custom-token endpoints disagree).
- `AccountInfo::has_sso_link()` keys off provider id `oidc.workos` — i.e. **WorkOS** is Warp's enterprise SSO provider. `needs_sso_link` on the user is computed server-side.

This crate contains no networking; it is the wire format the `AuthSession`
(`warp_server_client`) deserializes Firebase responses into.

---

## 4. `graphql` (`warp_graphql`) + `warp_graphql_schema` — typed cloud API

- **`warp_graphql_schema`** (`src/lib.rs`) is just `#[cynic::schema("warp-server")] pub mod schema {}` plus a `build.rs`; it registers the SDL so the [cynic] codegen can produce typed query structs.
- **`graphql`** is the big typed-operations crate (`crates/graphql/src/`):
  - `client.rs` — `Operation` trait, `RequestOptions`, and **`build_graphql_request`** which constructs the URL `"{server_root}/graphql/v2?op={name}"` (`client.rs:91`), attaches `bearer_auth`, timeout, headers, optional `path_prefix`. `send_graphql_request` classifies failures into `GraphQLError::{RequestError, StagingAccessBlocked, IapChallengeBlocked, HttpError, ResponseError}` and calls `http_client::iap::is_iap_challenge` to detect stale IAP creds. `get_request_context()` populates client/OS context.
  - `api/` — typed models for every cloud noun: `user`, `workspace`, `workflow`, `notebook`, `folder`, `billing`, `ai`, `object`, `object_permissions`, `mcp_gallery_template`, `experiment`, etc.
  - `api/mutations/` — the *write* surface and a direct catalogue of "things that need an account": `create_anonymous_user`, `mint_custom_token`, `generate_api_key` / `expire_api_key`, `create_team` / `send_team_invite_email` / `set_team_member_role` / `remove_user_from_team`, `set_user_is_onboarded`, `update_user_settings`, `create_agent_task`, `issue_task_identity_token`, `delete/update_managed_secret`, `set_object_link_permissions`, `purchase_addon_credits`, etc.
  - `api/subscriptions/get_warp_drive_updates.rs` + `subscriptions/mod.rs` — **`start_graphql_streaming_operation`** drives a `graphql-transport-ws` subscription over the **RTC** websocket (`rtc.app.warp.dev`). `init_payload` carries auth after the handshake; `handshake_headers` carry IAP `Proxy-Authorization` for staging.
  - `object_permissions.rs` exports `OwnerType` / `AccessLevel`, reused by `warp_server_auth` and the cloud-object crates.

`graphql` is `cfg`-split for `wasm` vs native; it is the *only* place GraphQL URLs
are assembled.

---

## 5. `warp_server_client` — the authenticated transport client

`crates/warp_server_client/src/` is the runtime that turns `Credentials` into
authenticated requests. Modules: `base_client`, `auth` (+`session`), `public_api`,
`graphql_helpers`, `drive`, `iap`, `network_logging`, `ids`.

- **`base_client.rs` — `BaseClient`**: owns the shared `http_client::Client`, the `Arc<AuthState>`, an `AuthEvent` channel, the `AuthSession`, and request-decoration policy. Adds Warp-specific headers including `X-Warp-Ambient-Workload-Token` (`AMBIENT_WORKLOAD_TOKEN_HEADER`), `X-Warp-Cloud-Agent-ID`, `X-Oz-Api-Source`, and `X-Warp-Experiment-Id`. `AmbientHeaderPolicy` controls per-request inheritance of ambient-agent headers. `GraphqlRoutingConfig`/`AuthenticatedGraphqlConfig` allow path-prefix routing and extra headers on session-authenticated ops. Optional `IapTokenProvider` for staging.
- **`auth/mod.rs` — `AuthClient` trait + `AuthClientImpl`**: the protocol-level auth API. Methods: `create_anonymous_user`, `get_or_refresh_access_token`, `fetch_user`, `fetch_new_custom_token`/`on_custom_token_fetched`, `fetch_user_properties`, `get_user_settings`/`set_is_telemetry_enabled`/`set_is_crash_reporting_enabled`/`set_is_cloud_conversation_storage_enabled`/`update_user_settings`, `set_user_is_onboarded`, **`request_device_code`/`exchange_device_access_token`** (headless CLI/SDK login), `list/create/expire_api_key`, `list_agent_identities`. `SyncedUserSettings` is the server-stored privacy/telemetry triple. Errors funnel through `UserAuthenticationError`.
- **`auth/session.rs` — `AuthSession`**: the token engine.
  - **`get_or_refresh_access_token()`** (`session.rs:99`): **if `cfg!(feature="skip_login")` it `bail!`s immediately** ("failing all authenticated requests"). Otherwise: API key / Bearer → returned as-is; Firebase → refreshes via Firebase REST if the id token expires within 5 minutes (matching the Firebase SDK), emitting `AuthEvent::NeedsReauth` on `DeniedAccessToken` and `AuthEvent::AccessTokenRefreshed` on success; SessionCookie/Test → `NoAuth`.
  - **`exchange_credentials(LoginToken)`** → fresh `Credentials`.
  - **OAuth device flow**: `create_oauth_client()` (`session.rs:197`) builds a `BasicClient` with client id **`warp-cli`**, token URL `{server_root}/api/v1/oauth/token`, device URL `{server_root}/api/v1/oauth/device/auth`. The server mints a Firebase **custom** token (`FirebaseToken::Custom`) because Firebase has no native device flow.
  - **`fetch_auth_tokens()`**: POSTs to the Firebase REST endpoint; on failure falls back to Warp's **proxy** (`{server_root}/proxy/token`/`/proxy/customToken`) — so even Firebase auth can be tunneled through `app.warp.dev`.
  - **`AuthEvent`** enum: `StagingAccessBlocked`, `NeedsReauth`, `UserAccountDisabled`, `AccessTokenRefreshed`, `IapChallengeReceived` — the signals the app reacts to.
- **`public_api.rs`** — REST helpers (`HttpStatusError`); `iap.rs` — staging IAP integration; `network_logging.rs` — request logging.

---

## 6. The cloud-object (Warp Drive) stack

Four layered crates implement "Warp Drive" — the synced cloud store for
workflows, notebooks, folders, env vars, AI facts, MCP configs, cloud-agent
configs, etc. All of this **requires an account / network**.

- **`cloud_objects`** (`src/lib.rs`) — low-level, model-agnostic substrate: server ids (`ids.rs`: `ObjectUid`, `ServerId`, `SyncId`, `FolderId`, `HashedSqliteId`), `UserUid` (`auth`), object metadata/type/format (`cloud_object/`), and drive/sharing primitives (`drive/sharing.rs`: `SharingAccessLevel`; `drive/mod.rs`: `CloudObjectTypeAndId`). No persistence, no UI.
- **`cloud_object_models`** — the concrete typed models: `workflow`, `notebook`, `folder`, `mcp`, `env_vars`, `ai_fact`, `user_profile`, `preference`, `ai_execution_profile`, `cloud_agent_config`, `scheduled_ambient_agent`, `cloud_environment`, `json_model`, `server_cloud_object`, each with a `persistence` submodule.
- **`cloud_object_persistence`** (`src/lib.rs`) — model-agnostic **SQLite** infra: upsert/delete of object metadata & permissions, guest/link-sharing encode/decode (`encoded_permissions.rs`), and **refresh scheduling** (`refresh.rs`: `read_time_of_next_force_object_refresh` / `record_time_of_next_refresh`). This is the local mirror that gets reconciled against the server.
- **`cloud_object_client`** (`src/lib.rs`) — the GraphQL CRUD client for cloud objects (trait `#[automock]`-able), with `GuestIdentifier`, `ObjectActionType`, sharing/permission ops. Re-exports `cloud_object_models` + `cloud_objects`. This is what the app calls to read/write Drive objects; live updates arrive via the RTC subscription (§4).

---

## 7. WHERE LOGIN IS GATED (de-auth dossier feed)

There is **no single global login wall**; gating is distributed. Two distinct
layers:

### 7a. Seam-level gates (this subsystem)
1. **Baked server URLs + OSS override-block** — `warp_core/src/channel/config.rs:57-66` (`WarpServerConfig::production`) and `warp_core/src/channel/mod.rs:42` (`allows_server_url_overrides` = false for `Oss`). Marley cannot be repointed by flag; must patch config/channel.
2. **`skip_login` feature** — `warp_server_auth/Cargo.toml:11` → `warp_server_client/Cargo.toml:12` → `app/Cargo.toml:834`. When on: `AuthState::should_use_test_user()` injects a fake logged-in `User::test()` (`auth_state.rs`), **but** `AuthSession::get_or_refresh_access_token()` `bail!`s on every authenticated request (`session.rs:100`). So `skip_login` = "UI thinks you're logged in, all server calls fail" — a partial de-auth that disables Drive/AI rather than stubbing them.
3. **`get_or_refresh_access_token` / `exchange_credentials`** — every authenticated request funnels through here; the natural choke point to stub a synthetic token or short-circuit.
4. **`WARP_USER_SECRET`** compile-time env (`auth_state.rs:initialize`) — bakes a real persisted user into the binary, bypassing interactive login.

### 7b. Policy-level gates (the `app` crate, just above this subsystem)
- **`AuthState::is_anonymous_or_logged_out()`** is the master predicate. It is checked in **~30+ UI sites**, e.g. `app/src/workspace/view.rs` (lines 7749, 7809, 9409, 9495, 17548, 20765, 21045, 23269, 25727), `app/src/terminal/view.rs:13395,13508`, `app/src/settings/ai.rs:1659`, `app/src/resource_center/main_page.rs:513`, `app/src/cloud_object/model/view.rs:222`, `app/src/pane_group/mod.rs:2536`, `app/src/app_menus.rs:238`.
- **`app/src/auth/auth_manager.rs`** — `AuthManager` singleton owns login. `attempt_login_gated_feature(feature, variant)` (`auth_manager.rs:634`): if anonymous/logged-out, emits `AuthManagerEvent::AttemptedLoginGatedFeature` → shows the auth modal (`auth_view_modal.rs`). `LoginGatedFeature = &'static str` (a free-form feature tag). Other events: `SkippedLogin`, `NeedsReauth`, `LoginOverrideDetected`, `ReceivedDeviceAuthorizationCode`. `create_anonymous_user` / `initiate_anonymous_user_linking` implement the "use without an account, then upsell" funnel.
- **Auth UI** lives in `app/src/auth/` (`auth_view_modal.rs`, `auth_view_body.rs`, `login_error_modal.rs`, `needs_sso_link_view.rs`, `web_handoff.rs`).

> For the de-auth work: stubbing `is_anonymous_or_logged_out()` to `false`
> would satisfy the UI gates, but unless the server seam is also satisfied
> (real creds, or a stubbed backend) the gated features will still fail at the
> network layer. A clean "Ignibyte login seam" therefore needs both: (a) a
> credential injector at `AuthState`/`AuthSession`, and (b) a decision on whether
> Drive/AI features point at a replacement backend or are disabled.

---

## 8. What requires an account vs. works offline

| Capability | Needs account? | Notes |
|---|---|---|
| Local terminal / PTY, blocks, themes, settings | **No** | Local; not in these crates (see terminal subsystem). Runs while logged out. |
| Warp Drive (workflows, notebooks, folders, env vars, MCP configs) | **Yes** | `cloud_object_*` + GraphQL + RTC WS. Anonymous users get limited `PersonalObjectLimits`. |
| AI / Agent / cloud agents ("Oz") | **Yes** | `oz.warp.dev`, ambient workload tokens, `create_agent_task`. Gated by `is_anonymous_or_logged_out`. |
| Cloud conversation storage, telemetry/crash settings sync | **Yes** | `SyncedUserSettings` via GraphQL. |
| Session sharing | **Yes** | `sessions.app.warp.dev` WS. |
| Teams / invites / roles / billing | **Yes** | Team mutations. |
| Managed secrets | **Yes** | §10; server holds the public keyset, GCP federation. |
| Anonymous browsing | Network, no real login | `create_anonymous_user`, custom-token flow; feature-gated. |

Offline-hard-fail surfaces are exactly the GraphQL/RTC paths in §4–§6.

---

## 9. `remote_server` — remote (SSH) sessions & their own auth

`crates/remote_server/` is the client for Warp's **remote daemon** (remote dev,
codebase indexing, repo metadata over SSH). Highly relevant to Marley's
"spawn/write/read terminal sessions on a host" goal.

- `lib.rs` exposes a prost-generated `proto` module with `ClientMessage` envelopes: **`host_scoped`**, **`session_scoped`**, and notifications — i.e. the wire protocol for host- and session-level RPCs against a remote daemon.
- `transport.rs` / `ssh.rs` (`ssh` is non-wasm) / `manager.rs` / `setup.rs` — connection, SSH bootstrap, daemon lifecycle, glibc detection.
- **`auth.rs` — `RemoteServerAuthContext`**: an app-supplied callback bundle (`get_auth_token: Fn() -> BoxFuture<Option<String>>`, `remote_server_identity_key`, `user_id`, `user_email`, `crash_reporting_enabled`). Bearer tokens flow **only through protocol messages**, not headers; the daemon receives user identity via the `Initialize` handshake (for its own Sentry config). This decouples `remote_server` from app auth types.
- The token handed to the daemon feeds back into `AuthState` via `AuthState::apply_remote_server_auth_context()` / `set_remote_server_bearer_token()` (`warp_server_auth/src/auth_state.rs`), which sets `Credentials::Bearer` — an **externally-managed** credential that bypasses local refresh.

---

## 10. `managed_secrets` (+ `managed_secrets_wasm`)

`crates/managed_secrets/` encrypts user secrets (e.g. **Anthropic API keys** for
BYO-LLM, see `managed_secrets_wasm/src/lib.rs::encrypt_anthropic_api_key_secret`)
client-side before upload:
- `envelope.rs` (+`hpke_impl.rs`) — HPKE envelope encryption; `UploadKey::import_public_keyset` takes a server-published public keyset.
- `gcp.rs` — `GcpWorkloadIdentityFederation*`, `GcpCredentials` — federated GCP creds for server-side decryption/storage.
- `manager.rs` — `ManagedSecretManager` + **`ActorProvider`** trait (implemented by `AuthState`, §2) so secrets are bound to the logged-in UID.
- `client.rs` — `TaskIdentityToken`.
- `managed_secrets_wasm` is the `wasm-bindgen` shim exposing `encrypt_raw_secret` / `encrypt_anthropic_api_key_secret` to Warp-on-Web JS.

Secrets are inherently account-bound (the actor UID is part of the AAD), so this
is another account-required surface.

---

## 11. Networking primitives

- **`http_client`** (`crates/http_client/src/lib.rs`) — the shared `reqwest` wrapper (native + wasm). Adds Warp custom headers (`X-Warp-Client-Version`, `X-Warp-OS-*`, client role), SSE event streams (`reqwest_eventsource`), and the **IAP** module (`iap.rs`): `is_iap_challenge()` (detects GCP Identity-Aware Proxy 302/401/403 + `x-goog-iap-generated-response`), `proxy_auth_header()` (`Proxy-Authorization: Bearer …`), and the `IapTokenProvider` trait. IAP only matters on the **staging** server.
- **`http_server`** (`crates/http_server/src/lib.rs`) — a tiny local axum server the client runs on `PORT_BASE = 9277` ("Warp"), composed from injected `axum::Router`s. Used for local OAuth/web-handoff callbacks. A `SingletonEntity` model.
- **`websocket`** (`crates/websocket/src/lib.rs`) — unified websocket (`native.rs` via `async-tungstenite`, `wasm.rs`) implementing `graphql_ws_client`'s `WebsocketMessage`; backs the RTC/session-sharing subscriptions. Also has a `proxy.rs`.
- **`warp_web_event_bus`** (wasm-only) — emits `WarpEvent` to the host JS app, including **`LoggedOut`** and `SessionJoined`. This is the auth/session signalling channel for the embedded web build; must stay in sync with the upstream TS `WarpEvent` type.
- **`serve-wasm`** (`crates/serve-wasm/src/main.rs`) — a dev-only static file server (clap + tower-http `ServeDir`) for the wasm bundle; not part of the auth path.

---

## 12. `onboarding`

`crates/onboarding/` is the first-run slide deck (`src/slides/`:
`intro_slide`, `intention_slide`, `ai_setup_slide`, `ai_access_slide`,
`theme_picker_slide`, `project_slide`, `third_party_slide`, `agent_slide`,
`customize_slide`), plus the `callout` system and `telemetry`. It is **UI, not
auth**, but it is where the login prompt is *presented*: `lib.rs` exposes
`AI_FEATURES` / `WARP_DRIVE_FEATURES` strings used by the login slide's
**skip-login confirmation dialog** (`components/feature_optout_dialog.rs`). The
"sign in / skip" decision surfaced here drives `AuthManagerEvent::SkippedLogin`
in the app layer (§7b). `OnboardingIntention::{Terminal, AgentDrivenDevelopment}`
records whether the user wants the agentic surface — directly relevant to Marley's
custom panel framing.

---

## 13. Marley relevance — summary

- **De-auth (primary):** This subsystem *is* the auth boundary. The minimal seam to "remove mandatory login" is: (1) repoint or neutralize the baked URLs in `warp_core/src/channel/config.rs` and/or relax `Channel::Oss` in `channel/mod.rs`; (2) inject synthetic `Credentials` at `AuthState::initialize` / short-circuit `AuthSession::get_or_refresh_access_token`; (3) force `is_anonymous_or_logged_out()` → `false` in `app/src/auth`. The existing `skip_login` feature is a ready-made (but server-breaking) precedent and a good starting reference. A clean Ignibyte login seam should slot in at `AuthState`/`AuthSession` and at the `AuthManager` event boundary.
- **Session spawn/write/read:** `remote_server` already models host/session-scoped RPC (`proto::ClientMessage::{host_scoped, session_scoped}`) with a decoupled `RemoteServerAuthContext` — a strong template for Marley's "spawn a session, write to it, read output" panel, and a place where the Bearer-credential path (`Credentials::Bearer`, externally managed, no refresh) shows how to drive sessions without the Firebase login flow.
- **UI-surface expansion:** Real-time data for a workflow-visualization panel would ride the existing RTC subscription machinery (`graphql/src/api/subscriptions`, `websocket`); `cloud_object_client` + `cloud_object_models` (workflows, cloud-agent configs, scheduled ambient agents) are the data models a custom agentic panel would read/write. `http_server` (local axum :9277) and `warp_web_event_bus` are existing hooks for wiring a custom local UI/host-app bridge.
- **Rebrand:** Hardcoded `warp.dev` URLs, the `warp-cli` OAuth client id, the `wk-` API-key prefix, Firebase API key, and the `WarpEvent` web contract are all branding/identity touchpoints to inventory.

> **Round-3 note:** §13 is preserved as the round-1 analysis of *Warp's* seam. Its
> "de-auth" plan assumed Marley would fork Warp's client and stub the login; that is
> **not** what happened (see below). Read §13 as "how Warp's boundary works / how a
> future brain would model auth," not as a Marley to-do.

---

## Marley status @ M15

**Verdict: this subsystem is N/A for Marley today.** Marley is **local-first**. As of
M15 there is no cloud backend, no accounts, no login, and **zero networking
dependencies** in the workspace — a `grep` over every `crates/*/Cargo.toml` for
`reqwest | hyper | tungstenite | tonic | graphql | cynic | firebase | oauth2 | axum |
jsonwebtoken` returns **nothing**. None of the 18 crates documented in §2–§12 were
adopted; Marley rebuilt the terminal/cockpit from scratch rather than fork-and-stub
Warp's client stack, so there is **no Warp auth boundary to de-auth** (the round-1
premise of this doc). The `skip_login` / `AuthState` / `is_anonymous_or_logged_out`
levers in §7 describe Warp's code, not anything present in Marley.

**What Marley *did* build here** — `marley_remote` (`crates/marley_remote`,
`[Marley-original]`): a pure, clean-room `[user@]host[:port]` parser + `ssh` argv
builder. Marley opens a remote pane by **spawning the user's own `ssh` client as a
terminal** (the Terminal.app model): `ssh` owns *all* security — keys, `known_hosts`,
auth, passwords — and the crate holds **no secrets, no cloud relay, no bearer tokens,
no Firebase**. It binds to `marley_util::HostId` + the local-vs-remote path sum type.
This is the shipped realization of the remote-connection intake
(`docs/planning/intake/remote-connection-seam.md`) and it deliberately **does not
adopt Warp's `remote_server`** (a cloud daemon with its own `RemoteServerAuthContext`);
it reuses only the *pattern* (transport-agnostic remote target), as the intake's
INVENT/SKIP note directed.

### Maps to a *future* brain/cloud offering (adopt the pattern, re-invent the code)

If/when Marley ships the sold **"brain"** (hosted agent) or any cloud sync, these
Warp designs are the reference — but each must be **re-invented from permissive deps**,
never copied (AGPL; see licensing below):

| Warp design (this subsystem) | Why it maps to a future brain | Marley action |
|---|---|---|
| `AuthState` + `AuthSession` single token choke point, event-driven `NeedsReauth` (§2, §5) | A hosted brain needs one login + refresh seam | INVENT Marley's own; do **not** adopt `warp_server_auth`/`warp_server_client` |
| `graphql-transport-ws` realtime subscription over `websocket` (§4, §11) | Streaming agent output / shared sessions | Build on permissive `tungstenite` directly |
| Client-side envelope encryption of BYO-LLM keys before upload (`managed_secrets`, §10) | A brain that stores user API keys | Re-invent from a permissive HPKE crate |
| Transport-agnostic remote target (`remote_server` *pattern*; `marley_remote` today) | Agents-on-web / remote runners / device handoff (M5) | Extend `marley_remote` behind a transport trait — Marley-original |

### Warp-cloud specifics Marley will **not** need

- The baked `*.warp.dev` topology (`app` / `rtc` / `sessions` / `oz`), the hardcoded
  Firebase web API key, the `warp-cli` OAuth client id, the `wk-` API-key prefix, GCP
  IAP staging, and WorkOS SSO (§1, §3) — all Warp-hosted-backend identity.
- **Warp Drive** (`cloud_objects` / `cloud_object_models` / `_persistence` / `_client`,
  §6) as a synced cloud store — Marley's workflows/settings are **local files**
  (`marley_settings`, TOML). Any future sync is Marley's own store, not Warp Drive's schema.
- **Warp-on-Web** glue (`managed_secrets_wasm`, `warp_web_event_bus`, `serve-wasm`,
  §10–§11) — Marley is a native app; there is no web build and no `WarpEvent` JS contract.
- `remote_server`'s **cloud daemon + prebuilt-binary bootstrap over SSH** (§9) —
  superseded by the "spawn the user's `ssh`" model in `marley_remote`.

---

## Provenance & licensing

**All 18 crates in this subsystem are Warp's own source and are AGPL-3.0**
(`license.workspace = true` → `AGPL-3.0-only`; the MIT `warpui*` exceptions live in
subsystem 01, not here). **None were copied into Marley** — these docs are clean-room
*architecture analysis* (public design + behavior), and the Warp source is cloned only
into session scratch, never committed. This mirrors the provenance posture of
`docs/zed_architecture/`.

**Tag legend**
- `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]` — the crate encodes
  Warp's proprietary hosted-backend contract (auth, Warp Drive, managed secrets, remote
  daemon, Warp-on-Web). AGPL; do not adopt. Value to Marley = the *pattern* only, for a
  future brain, re-invented clean.
- `[permissive substance / AGPL file: standard http/websocket/firebase-client]` — the
  Warp crate *file* is AGPL, but its *substance* is a thin shim over a permissive
  ecosystem dep (`reqwest` / `axum` / `tungstenite` / `tower-http`, or standard Google
  Identity REST DTOs). Marley rebuilds the capability **from the permissive dep
  directly**, carrying no Warp IP — it does not adopt the AGPL file.
- `[Marley-original]` — Marley's own clean-room code (only `marley_remote` here).

| Crate doc | Provenance tag | Marley posture |
|---|---|---|
| `warp_server_auth` | `[Warp-derived/AGPL: backend glue]` | N/A; future-brain auth *pattern* only |
| `warp_server_client` | `[Warp-derived/AGPL: backend glue]` | N/A; re-invent for a future brain |
| `warp_graphql` | `[Warp-derived/AGPL: backend glue]` | N/A (typed ops vs `app.warp.dev`) |
| `warp_graphql_schema` | `[Warp-derived/AGPL: backend glue]` | N/A (Warp SDL) |
| `cloud_objects` | `[Warp-derived/AGPL: backend glue]` | N/A (Warp Drive; Marley uses local files) |
| `cloud_object_models` | `[Warp-derived/AGPL: backend glue]` | N/A |
| `cloud_object_persistence` | `[Warp-derived/AGPL: backend glue]` | N/A |
| `cloud_object_client` | `[Warp-derived/AGPL: backend glue]` | N/A |
| `warp_managed_secrets` | `[Warp-derived/AGPL: backend glue]` | future-brain BYO-key *pattern* only |
| `managed_secrets_wasm` | `[Warp-derived/AGPL: Warp-on-Web glue]` | won't need (native app) |
| `warp_web_event_bus` | `[Warp-derived/AGPL: Warp-on-Web glue]` | won't need (no web build) |
| `remote_server` | `[Warp-derived/AGPL: cloud daemon]` | superseded by `marley_remote` |
| `onboarding` | `[Warp-derived/AGPL: UI + login-skip funnel]` | not adopted |
| `http_client` | `[permissive substance / AGPL file: reqwest]` | rebuild from `reqwest` if a brain needs it |
| `http_server` | `[permissive substance / AGPL file: axum]` | rebuild from `axum` if needed |
| `websocket` | `[permissive substance / AGPL file: tungstenite]` | rebuild from `tungstenite` if needed |
| `firebase` | `[permissive substance / AGPL file: std Google Identity DTOs]` | re-model DTOs if Firebase is ever used |
| `serve-wasm` | `[permissive substance / AGPL file: tower-http ServeDir]` | won't need (native, no wasm bundle) |
| *(`marley_remote`)* | `[Marley-original]` | **shipped** — the local ssh remote seam |

> `jsonrpc` and `warp_channel_config` appear in this subsystem's reading list but their
> crate docs are filed under subsystems **04** (agent-ai-mcp) and **06**
> (platform-settings-infra) respectively, so their provenance tags belong to those
> reviewers and are not duplicated here.

### AGPL exposure — flag for any future hosted service

AGPL-3.0's **§13 (network-use / remote-interaction)** is the sharp edge, and it lands
*exactly* on this subsystem because this is the network/cloud seam. Unlike GPL-2.0,
AGPL triggers the copyleft obligation when users **interact with the software over a
network** — even if no binary is ever distributed and the code runs only server-side.
Consequently, **any future Marley hosted service (the sold "brain") that incorporates
even one line of Warp-derived (AGPL) code from this subsystem would be obligated to
offer its complete corresponding source to every network user** — fatal to a
proprietary offering.

**Rule:** the brain/cloud layer must stay **clean of all Warp-derived AGPL code** from
§2–§12. Re-invent the auth / realtime-subscription / managed-secrets patterns from
permissive deps behind a hard module boundary, keep `[Marley-original]` and
`[permissive]` code strictly separated from any `[Warp-derived/AGPL]` lineage, and gate
release on legal review. This is the same open-core boundary the Zed reference draws
(GPL editor/terminal vs proprietary brain) — with AGPL making the *server-side* boundary
even less forgiving than GPL would be. Today the risk is zero (no Warp code, no service);
the flag is for the milestone that introduces a hosted brain.
