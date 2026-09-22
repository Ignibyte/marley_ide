# managed_secrets_wasm

> Per-crate reference (Marley round 2) — crate dir `crates/managed_secrets_wasm`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp-on-Web glue — not adopted]`:** a `wasm-bindgen` shim exposing envelope encryption to the browser dashboard (Warp-on-Web). Marley is a **native app with no web build** → **won't need**. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`) |
| Internal deps | 1 |
| Used by | 0 (leaf — built as a wasm artifact) |

## Purpose

A thin `wasm-bindgen` shim that exposes [`warp_managed_secrets`](./warp_managed_secrets.md)'s **client-side envelope encryption** to JavaScript. It exists so Warp's web surfaces (the website / dashboard where a user pastes an API key to store as a managed secret) can seal that secret against the backend's public keyset entirely in the browser — the plaintext never leaves the page un-encrypted. It is `crate-type = ["cdylib"]`, i.e. compiled to a `.wasm` module rather than linked into the native app.

## Key types, modules & public API

Single file `src/lib.rs`. All exports are `#[wasm_bindgen]` free functions returning `Result<String, JsValue>` (the base64 sealed ciphertext, or a JS error):

- `start()` — `#[wasm_bindgen(start)]` lifecycle hook; calls `init_envelope()` exactly once on module instantiation.
- `encrypt_raw_secret(public_key_base64, actor_uid, secret_name, secret_value)`
- `encrypt_anthropic_api_key_secret(public_key_base64, actor_uid, secret_name, api_key)`
- `encrypt_anthropic_bedrock_api_key_secret(..., aws_bearer_token_bedrock, aws_region)`
- `encrypt_anthropic_bedrock_access_key_secret(..., aws_access_key_id, aws_secret_access_key, aws_session_token: Option<String>, aws_region)`
- `encrypt_openai_api_key_secret(..., api_key, base_url: Option<String>)`

Private helper `do_encrypt(...)` imports the public keyset via `UploadKey::import_public_keyset` and calls `encrypt_secret`, mapping errors to `JsValue`. Each public function just constructs the matching `ManagedSecretValue` variant and delegates.

## Depends on (internal)

- [`warp_managed_secrets`](./warp_managed_secrets.md) — supplies `ManagedSecretValue`, `UploadKey`, and `init_envelope`; this crate is purely its wasm-facing wrapper.

## Used by (internal dependents)

None inside the workspace. It is a build target consumed by web/JS code outside this repo.

## Related crates

- [`warp_managed_secrets`](./warp_managed_secrets.md) — the real implementation; read it first.
- [`websocket`](./websocket.md), [`http_client`](./http_client.md) — other crates with wasm targets, illustrating the repo's native/wasm split.

## Marley relevance

**Classification: REMOVE.** This is a website-only artifact for Warp's hosted managed-secrets dashboard, which Marley does not run. None of the four goals touch it: it is not part of the desktop app surface (goal 1), not in the session path (goal 2), and the de-auth/login-stub goal (goal 3) eliminates managed secrets entirely. Deleting it is low-risk because it has **zero internal dependents** — nothing in the workspace will fail to build. If `warp_managed_secrets` is later kept as a local vault, this wrapper can be revived, but for the initial offline Marley boot it is dead weight and should be dropped from the workspace members. (Rebrand, goal 4: moot once removed.)

## Notes / gotchas

- `cdylib` only — it produces a `.wasm`/JS-glue artifact, not a Rust rlib; nothing in the native build links it.
- The crate name lacks the `warp_` prefix (it is `managed_secrets_wasm`, not `warp_managed_secrets_wasm`), an inconsistency to note when scripting a bulk de-Warp rename.
- Inherits the pinned `rand 0.9` / hpke crypto constraints transitively from `warp_managed_secrets`.
