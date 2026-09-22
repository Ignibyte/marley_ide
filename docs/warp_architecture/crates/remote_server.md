# remote_server

> Per-crate reference (Marley round 2) — crate dir `crates/remote_server`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance `[Warp-derived/AGPL: Warp's cloud/backend glue — not adopted]`:** Warp's remote-dev **cloud daemon** (protobuf RPC + `RemoteServerAuthContext` bearer tokens + prebuilt-binary SSH bootstrap). **Superseded** in Marley by `marley_remote` `[Marley-original]` — a clean-room ssh-argv builder that spawns the **user's own `ssh` client** (no daemon, no cloud, no bearer tokens). Only the transport-agnostic *pattern* is reused. See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [Cloud, Auth & Networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no own LICENSE marker) |
| Internal deps | 5 |
| Used by | 2 |

## Purpose

The **client-side** of Warp's SSH remote-development feature. It lets the local Warp app connect to a remote host over SSH, install/verify a remote daemon binary there, and drive remote sessions: opening/reading/writing files, running ripgrep searches, computing git diff state, committing/pushing/creating PRs, and keeping a remote codebase index in sync — all over a length-prefixed protobuf protocol tunneled through an SSH `ControlMaster` connection. This crate is the app-resident manager + protocol + transport; the daemon process it talks to is shipped as a separate prebuilt artifact (see Setup below), not built here.

The crate is transport-agnostic by design: SSH specifics are isolated behind a `RemoteTransport` trait so alternative transports (Docker exec, in-process test transport) can be slotted in.

## Key types, modules & public API

`src/lib.rs` re-exports `HostId` and defines the generated `proto` module via `include!(concat!(env!("OUT_DIR"), "/remote_server.rs"))` (compiled by `build.rs` from `proto/remote_server.proto` + `proto/diff_state.proto`).

- **`mod proto`** — prost-generated message types plus hand-written `ClientMessage` constructors: `ClientMessage::host_scoped(request_id, inner)`, `::session_scoped(request_id, inner)`, `::notification(inner)` that wrap inner messages in the `HostScopedRequest`/`SessionScopedRequest`/`Notification` envelopes.
- **`manager`** — the heart of the crate. `pub struct RemoteServerManager` (a GPUI `Model`, `::new(ctx: &mut ModelContext<Self>)`) owns clients keyed by host and tracks sessions. Notable methods: `connect_session`, `deregister_session`, `client_for_host`/`client_for_session`, `find_connected_session`, `send_host_request`, `send_host_scoped_request`, `start_ripgrep_search`, `abort_host_request`, `check_binary`, `install_binary`, `rotate_auth_token`, `get_diff_state`/`get_branches`/`unsubscribe_diff_state`/`discard_files`, `git_commit_chain`/`git_push_branch`/`git_create_pr`/`git_generate_commit_message`/`git_get_committed_branch_files`, `ensure_codebase_indexed`/`resync_codebase`/`trigger_codebase_incremental_sync`/`drop_codebase_index`, `navigate_to_directory`, `load_remote_repo_metadata_directory`, `update_git_status`. Also exposes the `HostRequestHandle` helper (`write_file`, `delete_file`, `save_buffer`, `open_buffer`, `read_file_context`, `git_*`, …) and enums `RemoteServerInitPhase`, `RemoteServerOperation`, `RemoteServerErrorKind`, `RemoteCodebaseIndexUpdateOperation`, `RemoteServerExitStatus`, event type `RemoteServerManagerEvent`.
- **`client` (`client/mod.rs`)** — `pub struct RemoteServerClient` wrapping one daemon connection: `new`, `from_child_streams`, `initialize`, `authenticate(&self, auth_token)`, `update_preferences`, `notify_session_bootstrapped`, `navigate_to_directory`, `load_repo_metadata_directory`, `get_diff_state`, `open_buffer`/`send_buffer_edit`/`close_buffer`, `run_command`, `send_host_scoped`, `abort_request`, `is_disconnected`. Errors via `pub enum ClientError`. Plus `spawn_stderr_forwarder`.
- **`auth`** — `pub struct RemoteServerAuthContext::new(get_auth_token, remote_server_identity_key, user_id, user_email, crash_reporting_enabled)`. App-supplied callback bundle so the crate stays decoupled from app auth types; `get_auth_token()` returns a `BoxFuture<'static, Option<String>>`, `remote_server_identity_key()` selects the daemon's socket/PID dir. Tokens travel only inside protocol messages; identity keys are non-secret partition keys.
- **`protocol`** — length-prefixed protobuf framing. `MAX_MESSAGE_SIZE = 64 MiB`, `pub struct RequestId`, `pub enum ProtocolError`, and async `read_message`/`write_message`/`read_client_message`/`write_client_message`/`read_server_message`/`write_server_message`.
- **`transport`** — `pub trait RemoteTransport: Send + Sync + Debug` (returns boxed futures for object-safety; manager stores `Arc<dyn RemoteTransport>` for reconnect), plus `pub struct Connection`, `pub enum InstallSource`, `InstallOutcome`, `SetupStage`, `UserFacingError`, `ControlPath`, and transport `Error`.
- **`ssh`** (cfg `not(wasm)`) — concrete SSH transport helpers: `ssh_args`, `run_ssh_command`, `run_ssh_script`, `scp_upload`, `stop_control_master`, `SshCommandError`.
- **`setup`** — remote install/version logic: `RemoteServerSetupState`, `PreinstallCheckResult`/`PreinstallStatus`/`UnsupportedReason` (driven by the bundled `preinstall_check.sh` via `PREINSTALL_CHECK_SCRIPT`), `RemotePlatform`/`RemoteOs`/`RemoteArch`, `parse_uname_output`, path helpers (`remote_server_dir`, `remote_server_binary`, `daemon_socket_name`, …), `install_script`, `download_tarball_url`, and `setup::glibc::{GlibcVersion, RemoteLibc}`.
- **`host_id`** — re-exports `warp_core::HostId`.
- **`host_response`** — parsers that turn raw host-scoped `ServerMessage`s into `Result<(), String>` so nested per-op errors are never read as success: `write_file_result`, `save_buffer_result`, `delete_file_result`, `discard_files_result`.
- **`codebase_index_proto`**, **`repo_metadata_proto`** — proto adapters for the remote codebase-index and repo-metadata features.

## Depends on (internal)

- [command](./command.md) — running/representing commands (remote command execution model).
- [repo_metadata](./repo_metadata.md) — git repo metadata types synced to/from the remote (`load_remote_repo_metadata_directory`).
- [warp_core](./warp_core.md) — core app model; source of `HostId`, session/path types, GPUI model context.
- [warp_util](./warp_util.md) — shared utilities incl. `path::hashed_asset_url` and standardized paths.
- [warpui_core](./warpui_core.md) — UI-core async primitives (e.g. `r#async::BoxFuture` used by `RemoteServerAuthContext`). *(MIT crate.)*

## Used by (internal dependents)

- [warp](./warp.md) — the umbrella app crate; wires the manager into the running app.
- [warp_files](./warp_files.md) — file-tree/editor features that read/write files through `RemoteServerManager`/`RemoteServerClient`.

## Related crates

- [repo_metadata](./repo_metadata.md) and [command](./command.md) — the data/command models marshaled over the wire.
- [warp_files](./warp_files.md) — primary consumer for remote file operations.
- [warp_web_event_bus](./warp_web_event_bus.md) — sibling in the cloud/auth/net subsystem (web-side); contrast with this native-side SSH path.
- The out-of-repo **remote-server daemon** artifact (the server side of this protocol).

## Marley relevance

**Classification: KEEP (defer), with a STUB carve-out for auth.**

This is a large, self-contained feature (5 internal deps, ~12 modules, generated protos, an out-of-repo daemon). It is **orthogonal to the four core Marley goals** — custom panel, local session spawn/read/write, login stub, rebrand — which all target the *local* app. So for the initial Marley milestone, **KEEP it as-is and do not port/enable it**; treat remote-dev as an optional later feature, not part of the offline boot path.

Concrete guidance tied to the goals:
- **De-auth + login stub (goal 3):** the only auth coupling is `RemoteServerAuthContext::new(...)`, which takes app-supplied callbacks. For an offline Marley, **STUB** `get_auth_token` to return `async { None }` and feed placeholder `user_id`/`user_email` with `crash_reporting_enabled = false`. Because auth is injected (not imported), no code in this crate needs changing — the stub lives at the call site in `warp`/app wiring. This keeps Marley booting without a Warp account even if the crate is compiled in.
- **Session spawn/read/write (goal 2):** Marley's MVP is *local* sessions; this crate is *remote* sessions. Do **not** route the local panel through `RemoteServerManager`. Reuse the protocol patterns conceptually only.
- **De-Warp rebrand (goal 4):** the daemon download URL (`download_tarball_url`), `remote_server_dir`/path helpers, and `proto/remote_server.proto` all hardcode Warp naming/artifact layout. Renaming touches the **out-of-repo daemon** and its release pipeline, which we don't fork — so **defer** any rename until/unless we stand up our own daemon artifact.
- **Custom panel (goal 1):** unrelated; no work here.

Net: **KEEP but gate off** for v1; stub auth at the wiring layer; revisit remote-dev (and its rebrand cost) only after the local milestone ships.

## Notes / gotchas

- **Generated code:** `build.rs` runs `prost_build::compile_protos` over `proto/remote_server.proto` and `proto/diff_state.proto`; the `proto` module is `include!`-ed from `OUT_DIR`. A missing `protoc`/prost-build failure breaks the build. `#[allow(clippy::large_enum_variant)]` on the generated module.
- **Out-of-repo daemon:** the *server* binary this crate talks to is **not** in this repo — it's a prebuilt tarball fetched via `download_tarball_url(platform)` and installed by `install_script`/`preinstall_check.sh`. The protocol contract spans repos.
- **Platform/cfg gating:** `ssh`, much of `manager`, and `transport`'s SSH bits are `#[cfg(not(target_family = "wasm"))]`; on `wasm` the crate pulls `getrandom` with the `js` feature and drops the native transport. SSH itself relies on a system `ssh` binary and `ControlMaster` sockets — Unix-centric.
- **GPUI-coupled:** `RemoteServerManager` is a GPUI `Model` constructed from `&mut ModelContext<Self>`; it can't be used outside the app's GPUI runtime without that context.
- **Error-handling discipline:** `host_response.rs` exists specifically because host-scoped ops nest operation errors inside success-shaped responses; an exhaustiveness test forces every new request variant to be classified. Preserve that pattern when extending.
- Large surface: `manager.rs` alone is 3k+ lines — budget accordingly if porting.
