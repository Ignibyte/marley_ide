# warp_managed_secrets

> Per-crate reference (Marley round 2) — crate dir `crates/managed_secrets`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** HPKE envelope encryption of BYO-LLM keys + GCP workload-identity federation to Warp's secret backend. Marley is local-first → **N/A**; the *client-side-encrypt-before-upload pattern* maps to a future brain, re-invented from a permissive HPKE crate. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`) |
| Internal deps | 4 |
| Used by | 3 |

## Purpose

Client-side library for Warp's **managed secrets** feature: secrets (API keys, cloud credentials) that the user stores with Warp's backend and that get injected into agent/task runtimes without the plaintext ever living on disk locally. The crate provides:

- A typed `ManagedSecretValue` model for the kinds of secret Warp understands (raw strings plus structured provider credentials — Anthropic API key, Anthropic Bedrock API key / access key, OpenAI API key).
- **Envelope encryption** (`envelope` module) using a pure-Rust [HPKE](https://www.rfc-editor.org/rfc/rfc9180) (RFC 9180, X25519) implementation so a secret can be sealed against the server's public key before upload — the client encrypts; only the backend can decrypt.
- A `ManagedSecretManager` GPUI singleton that talks to the GraphQL backend to create/update/delete/list secrets and to issue short-lived task identity and **GCP Workload Identity Federation** tokens.
- A `gcp` module that materializes federated GCP credentials onto disk (temp files) and exposes them as environment variables for spawned processes.

## Key types, modules & public API

Re-exported from `src/lib.rs`:

- `manager::ManagedSecretManager` — the entry point. `new(...)`, plus async ops returning futures: `create_secret`, `update_secret`, `delete_secret`, `list_secrets`, `get_task_secrets` (returns `HashMap<String, ManagedSecretValue>`), `issue_task_identity_token`, `issue_gcp_workload_identity_federation_token`. Implements `warpui_core::Entity` + `SingletonEntity`.
- `manager::ActorProvider` — trait the host implements to supply the current actor/identity.
- `secret_value::ManagedSecretValue` — enum of secret kinds with constructors `raw_value`, `anthropic_api_key`, `anthropic_bedrock_api_key`, `anthropic_bedrock_access_key`, `openai_api_key`, and `secret_type()`.
- `envelope::{UploadKey, init as init_envelope, EnvelopeError}` — `init_envelope()` registers the tink/HPKE primitives (must be called once before use); `UploadKey::import_public_keyset(base64)` then `UploadKey::encrypt_secret(actor_uid, secret_name, &ManagedSecretValue)` performs the seal.
- `client::{TaskIdentityToken, ManagedSecretsClient, ManagedSecretConfigs, SecretOwner, IdentityTokenOptions}` — the backend-facing client trait + value types.
- `gcp::{GcpCredentials, GcpFederationConfig, GcpWorkloadIdentityFederationToken, GcpWorkloadIdentityFederationError, PrepareGcpCredentialsError}` — `GcpCredentials::federated(...)` builds creds, `.env_vars()` exposes them for child processes, `.cleanup()` removes the temp files.

Module map: `client.rs`, `envelope.rs` (+ `envelope/hpke_impl.rs`), `gcp.rs`, `manager.rs`, `secret_value.rs`.

## Depends on (internal)

- [`warp_core`](./warp_core.md) — channel/config and core runtime plumbing (e.g. error reporting, execution context).
- [`warp_graphql`](./warp_graphql.md) — generated GraphQL operations used by the manager to talk to the secrets backend.
- [`warp_isolation_platform`](./warp_isolation_platform.md) — the task/agent isolation runtime that consumes injected secrets and identity tokens.
- [`warpui_core`](./warpui_core.md) — the GPUI `Entity`/`SingletonEntity`/`ModelContext` model framework (MIT).

## Used by (internal dependents)

- [`managed_secrets_wasm`](./managed_secrets_wasm.md) — wraps the envelope encryption for browser/JS callers.
- [`warp`](./warp.md) — the main app binds the manager as a singleton.
- [`warp_server_auth`](./warp_server_auth.md) — server-side auth consumes the secret/identity types.

## Related crates

- [`http_client`](./http_client.md) — its `iap` module handles the GCP IAP challenges that pair with the federated GCP credentials produced here.
- [`warp_server_auth`](./warp_server_auth.md) and [`warp_graphql`](./warp_graphql.md) — the auth + transport layer this rides on.

## Marley relevance

**Classification: STUB (lean toward REMOVE later).** This crate is pure Warp cloud surface — it only has value when there is a Warp backend issuing keysets and federation tokens. None of the four Marley goals need it: the custom panel (goal 1), local session spawn/read/write (goal 2), and the login stub (goal 3) all operate offline against a local Claude/agent runtime. For the **de-auth + login stub** goal, short-circuit `ManagedSecretManager` so `list_secrets`/`get_task_secrets` return empty and `issue_*token` calls fail fast (or return a canned local token) — that keeps `warp_isolation_platform` and any agent path from blocking on a backend that no longer exists. Defer full removal because `warp_isolation_platform`, `warp`, and `warp_server_auth` all reference its types; ripping it out now means touching the isolation runtime. For the **de-Warp rebrand** goal the `warp_` package prefix should eventually become `marley_managed_secrets`, but rename is low priority given it is slated to be stubbed. If Marley later wants its own secret vault, the `ManagedSecretValue` + HPKE envelope design is a reusable, self-contained pattern worth keeping even after the cloud manager is gutted.

## Notes / gotchas

- Out-of-repo crypto stack: `tink-core`/`tink-proto`/`tink-hybrid` 0.3 plus `hpke` 0.13 (default-features off, `alloc` + `x25519`). `rand` is **pinned to 0.9** (not the workspace 0.8) to satisfy hpke's `rand_core` 0.9 requirement — a version-skew trap if you bump workspace deps.
- `init_envelope()` must run before any encrypt call; the wasm wrapper calls it from its `#[wasm_bindgen(start)]`.
- wasm target pulls `getrandom` 0.3 with the `wasm_js` feature.
- `gcp::GcpCredentials::federated` writes credential temp files and hands their paths via env vars to spawned tasks; call `cleanup()` to avoid leaking them.
- Many manager methods return `impl Future<... > + use<>` (edition-2024 precise capturing) rather than `async fn`.
