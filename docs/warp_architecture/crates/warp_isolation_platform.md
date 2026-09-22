# warp_isolation_platform

> Per-crate reference (Marley round 2) — crate dir `crates/isolation_platform`. Marley is Ignibyte's fork of Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL]` — Docker/K8s/namespace sandbox reading `WARP_*` env; not ported (relevant later for remote/agent-session isolation). See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (no in-crate LICENSE marker; inherits workspace `license`) |
| Internal deps | 2 (`command`, `warp_core`) |
| Used by | 3 (`warp`, `warp_managed_secrets`, `warp_server_client`) |

## Purpose

Detects whether the running Warp process is inside a sandbox / hosted-agent **isolation platform** (Docker container, Warp-hosted Docker Sandbox, Kubernetes pod, or a [Namespace](https://namespace.so) instance) and, where the platform supports it, **issues a workload-identity token** so the agent can authenticate to Warp's backend without a human login. It exists to give Warp-hosted and self-hosted background agents a machine identity. Per the `Cargo.toml` description: *"Utilities for running Warp within an isolated sandbox, such as a Docker container or Github Actions."*

## Key types, modules & public API

`src/lib.rs` is the public surface; the per-platform logic lives in private `#[cfg(not(target_family = "wasm"))]` modules.

- `pub enum IsolationPlatformType` — `Docker`, `DockerSandbox`, `Kubernetes`, `Namespace` (serde `snake_case`).
- `pub struct WorkloadToken { pub token: String, pub expires_at: Option<DateTime<Utc>> }`.
- `pub fn detect() -> Option<IsolationPlatformType>` — memoized via `OnceLock`. Precedence: `WARP_ISOLATION_PLATFORM` env (parsed by `platform_from_env`) → `namespace::is_in_namespace_instance()` → `kubernetes::is_in_kubernetes()` → `docker::is_in_docker()`. Returns `None` for the `Channel::Integration` test channel.
- `pub async fn issue_workload_token(duration: Option<Duration>) -> Result<WorkloadToken, IsolationPlatformError>` — dispatches to `docker_sandbox::issue_workload_token` / `namespace::issue_workload_token`, else falls back to the generic `WARP_WORKLOAD_TOKEN` env var (`read_generic_workload_token`).
- `pub enum IsolationPlatformError` (`thiserror`) — `NoIsolationPlatformDetected`, `GenericWorkloadTokenMissing`, `CommandUnavailable`, `CommandFailed`, `Other(anyhow::Error)`.
- Internal modules: `namespace` (shells out to the `nsc` CLI via `command::r#async::Command`, parses the returned JWT `exp` claim in `parse_jwt_expiration`), `docker`, `docker_sandbox`, `kubernetes` (each a thin detection/token helper).
- Env constants: `WARP_ISOLATION_PLATFORM_ENV`, `WARP_WORKLOAD_TOKEN_ENV`.

## Depends on (internal)

- [`command`](./command.md) — `command::r#async::Command` to shell out to the `nsc` CLI (and other platform tools) for token issuance.
- [`warp_core`](./warp_core.md) — `warp_core::channel::{Channel, ChannelState}` for channel gating (skips detection on the integration channel) and `ChannelState::workload_audience_url()` as the token audience.

## Used by (internal dependents)

- [`warp`](./warp.md) — top-level app; wires isolation detection into boot / auth.
- [`warp_managed_secrets`](./warp_managed_secrets.md) — uses the workload token to fetch managed secrets.
- [`warp_server_client`](./warp_server_client.md) — presents the workload token when authenticating to the backend.

## Related crates

- [`warp_core`](./warp_core.md) — channel + paths primitives this crate sits on top of.
- [`command`](./command.md) — subprocess wrapper used for `nsc`.
- [`warp_server_client`](./warp_server_client.md) / [`warp_managed_secrets`](./warp_managed_secrets.md) — the consumers that turn a `WorkloadToken` into real auth.

## Marley relevance

**Classification: STUB (or REMOVE).** This crate is pure Warp-hosted-agent infrastructure: its whole reason for being is letting Warp's *cloud* sandboxes authenticate to *Warp's* backend. None of the four Marley goals — custom panel, session spawn/write/read, de-auth + login stub, de-Warp rebrand — need it. For goal (3) de-auth, the cleanest move is to make `detect()` always return `None` and `issue_workload_token()` always return `NoIsolationPlatformDetected` (a one-line stub), so the three dependents fall straight through to the offline/login-stub path and never try to call `nsc` or reach a Warp audience URL. Because it has only 3 dependents and a tiny surface, full REMOVE is viable once those callers are stubbed. Rename `WARP_*` env vars only if we keep it; otherwise the rebrand (goal 4) is moot. Low priority either way.

## Notes / gotchas

- **Out-of-repo runtime dep:** Namespace token issuance execs the external `nsc` binary; absent it returns `CommandUnavailable`. Detection also probes the filesystem path `/var/run/nsc/token.json` and the `NSC_TOKEN_FILE` env var.
- The `nsc` audience comes from `ChannelState::workload_audience_url()` — a Warp-controlled URL; keep that in mind for goal (3).
- Almost everything is gated `#[cfg(not(target_family = "wasm"))]`; on the wasm/web target the crate degrades to "no platform, no token."
- `detect()` results are memoized for the process lifetime via `OnceLock`, so env changes after first call are ignored.
