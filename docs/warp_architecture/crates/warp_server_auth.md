# warp_server_auth

> Per-crate reference (Marley round 2). Crate dir: `crates/warp_server_auth`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** client-side auth state / `Credentials` / Firebase for Warp's hosted backend. Marley is local-first (no login, zero networking deps) → **N/A today**; useful only as the *auth-seam pattern* for a future brain, re-invented from permissive deps. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (no own `LICENSE` marker; `license.workspace = true`) |
| Internal deps | 5 |
| Used by | 3 |

## Purpose

`warp_server_auth` owns the **client-side authentication state**: who is logged in, what credentials they hold, the anonymous identity, and how that state is persisted to (and restored from) secure storage. It is the in-memory source of truth for auth, exposed as a `warpui_core` singleton so any part of the app can ask "is the user logged in / what's their email / give me an access token." It does *not* perform the network login itself — that is [`warp_server_client`](./warp_server_client.md); this crate models and stores the result.

Feature flags of note: `skip_login`, `integration_tests`, `test-util` — these inject `Credentials::Test` / `Credentials::SessionCookie` and a fabricated `User::test()` so the app can boot fully authenticated without a real login.

## Key types, modules & public API

`src/lib.rs` exposes `pub mod anonymous_id, auth_state, credentials, user, user_uid`, and re-exports `AuthStateProvider` and `UserUid`. Constant `API_KEY_PREFIX = "wk-"`.

- **`auth_state`** (`src/auth_state.rs`):
  - `pub struct AuthState` — holds `RwLock<Option<User>>`, an `anonymous_id: Uuid`, `needs_reauth: AtomicBool`, and `RwLock<Option<Credentials>>`. Rich accessor surface: `initialize(ctx, api_key)` (resolution order: test user → API key → `WARP_USER_SECRET` env → persisted user), `is_logged_in()`, `is_anonymous_or_logged_out()`, `credentials()` / `set_credentials()`, `update_firebase_tokens()`, `get_access_token_ignoring_validity()`, `user_email()` / `display_name()` / `user_photo_url()`, `user_id() -> Option<UserUid>`, `is_api_key_authenticated()`, `principal_type()`, `is_service_account()`, `global_skills()`, `persist_action() -> PersistAction`, `apply_remote_server_auth_context()`, `set_remote_server_bearer_token()`, plus `new_for_test()` / `new_anonymous_for_test()` / `new_logged_out_for_test()`.
  - `pub enum PersistAction { Persist(Box<PersistedUser>), Remove, DoNothing }` — drives secure-storage writes.
  - `pub struct AuthStateProvider` — `impl SingletonEntity` (`warpui_core`); `new(Arc<AuthState>)` and `get() -> &Arc<AuthState>`. This is the global handle.
- **`credentials`** (`src/credentials.rs`): `pub enum Credentials { Firebase(FirebaseAuthTokens), ApiKey { key, owner_type }, Bearer(String), SessionCookie, Test }` with accessors `as_firebase()`, `as_api_key()`, `refresh_token()`, `api_key_owner_type()`. Doc distinguishes long-lived `LoginToken` (refresh) vs short-lived `AuthToken` (access).
- **`user`** (`src/user.rs` + `src/user/persistence.rs`): `pub struct User`, `pub struct UserMetadata`, `pub struct FirebaseAuthTokens` (`from_response`, `username_for_display`), `pub struct PersonalObjectLimits`, `pub enum AnonymousUserType`, `pub enum PrincipalType`, and `PersistedUser` (secure-storage shape).
- **`user_uid`** (`src/user_uid.rs`): `pub struct UserUid(lasso::Spur)` interned id; consts `TEST_USER_EMAIL`, `TEST_USER_UID`.
- **`anonymous_id`** (`src/anonymous_id.rs`): `pub fn get_or_create_anonymous_id(ctx) -> Uuid`.

## Depends on (internal)

- [`warp_core`](./warp_core.md) — `AppContext`/channel primitives, `ChannelState`, `report_error`, `Channel`.
- [`warp_graphql`](./warp_graphql.md) — `object_permissions::OwnerType` used in credentials/limits.
- [`warp_managed_secrets`](./warp_managed_secrets.md) — secure storage of persisted credentials.
- [`warpui_core`](./warpui_core.md) — `Entity`/`SingletonEntity`/`AppContext` model that `AuthStateProvider` plugs into. **(MIT)**
- [`warpui_extras`](./warpui_extras.md) — additional UI/runtime helpers. **(MIT)**

## Used by (internal dependents)

- [`warp_server_client`](./warp_server_client.md) — reads/writes `AuthState`, performs the network login that fills it.
- [`cloud_objects`](./cloud_objects.md) — needs the current user/credentials for object ownership.
- [`warp`](./warp.md) — the top-level app wires the `AuthStateProvider` singleton.

(Total internal dependents: 3.)

## Related crates

- [`warp_server_client`](./warp_server_client.md) — the action layer over this state.
- [`firebase`](./firebase.md) — supplies the wire types behind `FirebaseAuthTokens`.
- [`warp_managed_secrets`](./warp_managed_secrets.md) — where `PersistedUser` is stored.

## Marley relevance

**Classification: KEEP + STUB (the central de-auth lever).**

This is *the* crate to lean on for Marley goal **(3) de-auth + login stub** — and Warp already built the lever for us:

- **`skip_login` feature + `AuthState::new_for_test()` / `Credentials::Test`.** Building Marley with `skip_login` (and/or routing `initialize()` to return a fabricated logged-in or anonymous `User`) gives an app that boots fully "authenticated" with zero network and no Firebase. This is the lowest-risk offline-boot path: short-circuit `initialize()` to yield a `Test`/`SessionCookie` credential and a canned `User`.
- **(2) session spawn/write/read:** Sessions read `AuthState` for user id/anonymous id; a stub user keeps those calls total, so session paths don't fault.
- **(4) de-Warp rebrand:** `API_KEY_PREFIX = "wk-"`, `WARP_USER_SECRET` env var, and `TEST_USER_EMAIL = "test_user@warp.dev"` are Warp-branded strings → rename to Marley equivalents (low risk; mostly cosmetic). Package rename `warp_server_auth → marley_server_auth` has only 3 dependents → cheap when we do the sweep.

Do **not** REMOVE — `AuthStateProvider` is consulted broadly; removing it cascades. Keep the types, stub the resolution.

## Notes / gotchas

- **Multiple test/skip features interact:** `test_credentials()` picks `Credentials::Test` under `test`/`integration_tests`/`skip_login` but `Credentials::SessionCookie` under `test-util` alone — be deliberate about which feature you enable for the offline build.
- **`initialize()` reads the `WARP_USER_SECRET` env var** as a credential source — an undocumented backdoor worth auditing/renaming.
- **wasm gating:** a lot of persistence (`PersistedUser`, `UserMetadata`) is `cfg`-gated off wasm; `API_KEY_PREFIX` is `allow(dead_code)` on wasm.
- `edition = "2024"`. `UserUid` interns strings via `lasso` (a `Spur`), so ids are cheap to copy but tied to an interner.
