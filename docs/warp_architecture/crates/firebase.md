# firebase

> Per-crate reference (Marley round 2). Crate dir: `crates/firebase`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[permissive substance / AGPL file: standard Google Identity REST DTOs]`:** serde-only DTOs (no networking) for a **standard Google Identity Platform API**. The Warp *file* is AGPL, but the wire format is public; if Marley ever uses Firebase it would **re-model the DTOs cleanly**, not adopt this file. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (no own `LICENSE` marker; `license.workspace = true`) |
| Internal deps | 0 |
| Used by | 2 |

## Purpose

`firebase` is a tiny, dependency-light crate of **serde data models for Google Firebase / Identity Platform REST responses**. It exists so the auth/session code can deserialize the JSON returned by Google's `identitytoolkit` endpoints (`POST /v1/accounts:lookup`, `POST /v1/token`) without pulling in a heavy Firebase SDK. It carries *no logic and makes no network calls itself* — just the request/response shapes and a few accessors. Description in `Cargo.toml`: "Utilities for firebase client APIs".

Only two external deps: `anyhow` and `serde`.

## Key types, modules & public API

Everything lives in `src/lib.rs`:

- `pub struct FirebaseError { code: i32, message: String }` — the standardized Google API error format (implements `std::error::Error` + `Display`).
- `pub struct AccountInfo { local_id, photo_url, screen_name, .. }` — a user account record from `/v1/accounts:lookup`. Hides `display_name`, `email`, and a `Vec<ProviderUserInfo>` behind accessors:
  - `pub fn from_profile(firebase_uid, photo_url, display_name, email) -> Self`
  - `pub fn display_name(&self) -> Option<&str>` (falls back to provider info)
  - `pub fn email(&self) -> Result<&str>` (falls back to provider info; errors if absent)
  - `pub fn has_sso_link(&self) -> bool` (true when a provider is `oidc.workos`)
- `pub struct GetAccountInfoResponsePayload` with `pub fn user_account_info(self) -> Result<AccountInfo>` (pulls the first user).
- `pub enum GetAccountInfoResponse { Success(..), Error { error: FirebaseError } }` — `#[serde(untagged)]` success/error envelope for the lookup call.
- `pub enum FetchAccessTokenResponse { Success { expires_in, id_token, refresh_token }, Error { error } }` — `#[serde(untagged)]`, with `#[serde(alias = ...)]` to accept both camelCase and snake_case (the refresh-token and custom-token exchange endpoints disagree on casing).
- Private: `ProviderUserInfo` (display_name/email/provider_id).

## Depends on (internal)

None — leaf crate (`deps: []`).

## Used by (internal dependents)

- [`warp_server_client`](./warp_server_client.md) — uses `AccountInfo` / token responses when exchanging credentials and fetching the user during login/refresh.
- [`warp`](./warp.md) — the top-level app crate.

(Total internal dependents: 2.)

## Related crates

- [`warp_server_auth`](./warp_server_auth.md) — defines `Credentials::Firebase`, `FirebaseAuthTokens`; this crate supplies the wire types those are built from.
- [`warp_server_client`](./warp_server_client.md) — the `AuthClient` impl that performs the actual HTTP exchanges these types decode.

## Marley relevance

**Classification: STUB / candidate REMOVE.**

This crate is pure Firebase wire-format plumbing — exactly the Warp-cloud login machinery Marley wants to neutralize.

- **(3) de-auth + login stub:** These types only matter on the live Firebase login/refresh path. Once `warp_server_client` short-circuits auth (return a canned logged-in or anonymous user), nothing constructs a real `FetchAccessTokenResponse` or `GetAccountInfoResponse`. Easiest non-destructive move: leave the crate compiled (2 dependents reference it) but ensure no code path actually calls Google. It can later be **REMOVED** once those references are deleted in the de-auth sweep.
- It is *not* a rebrand target (no "Warp" branding, just Google's API shapes) and is irrelevant to UI (1) and sessions (2).

Recommendation: keep as-is for the offline-boot milestone (cheap, no network of its own), then delete in a cleanup pass after the auth stub lands.

## Notes / gotchas

- **No networking, no internal deps** — purely declarative serde models; safe to leave in place.
- **Casing duality is intentional:** `FetchAccessTokenResponse` uses `#[serde(alias)]` because Firebase's refresh-token vs custom-token endpoints return different field casing. Don't "clean this up" to a single casing.
- **SSO detection is hard-coded** to the `oidc.workos` provider id in `has_sso_link()` — a Warp-org-specific assumption.
- `edition = "2021"` (older than the `2024`-edition auth crates) and a hand-written `authors = ["Warp Team <dev@warp.dev>"]` line rather than `authors.workspace`.
