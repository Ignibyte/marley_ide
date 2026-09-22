# ipc

> Per-crate reference (Marley round 2). Crate dir: `crates/ipc`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance:** `[Warp-derived: AGPL-3.0]` generic request/response IPC over `[permissive: interprocess]` (UDS / named pipes). A transport *pattern* Marley can re-derive `[Marley-original]` if a control channel is needed; not carried today. See subsystem [Provenance & licensing](../subsystems/03-terminal-session-core.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [terminal-session-core](../subsystems/03-terminal-session-core.md) |
| License | AGPL v3 (inherits workspace `license = "AGPL-3.0-only"`; no own LICENSE marker) |
| Internal-deps count | 1 |
| Used-by count | 1 |

## Purpose

`ipc` is a small, generic **request/response IPC framework** for talking across process boundaries. A process stands up a `Server` hosting one or more typed `Service`s; other processes connect a `Client` and obtain typed `ServiceCaller`s that marshal a request, ship it over the wire, and await the matching response.

Its stated primary use (per `lib.rs` module docs) is communication between the **Warp app and third-party plugins running in a separate "plugin host" process**, but it is deliberately transport-agnostic and "designed generically to be extended to other use cases (such as the terminal server)". On native platforms it rides on the `interprocess` crate (Unix Domain Sockets on Unix, named pipes on Windows). WASM is stubbed and "currently unsupported."

## Key types, modules & public API

Re-exported from `src/lib.rs`:

- `Client`, `ClientError` (`src/client.rs`) — `Client::connect(connection_address, background_executor) -> Result<Client, ...>` opens the socket and spawns reader/writer tasks; `Client::wait_for_disconnect()` awaits teardown. Internally pairs each outbound `Request` (UUID `RequestId`) with a `PendingRequestInfo` to route the `Response` back.
- `Server`, `ServerBuilder` (`src/server.rs`) — builder pattern: `ServerBuilder::default().with_service(impl).with_fixed_address(String).build_and_run(executor) -> Result<(Server, ConnectionAddress)>`. The server listens, accepts connections, and dispatches each `Request` to the registered service by `ServiceId`.
- `Service`, `ServiceImpl`, `ServiceCaller`, `service_caller` (`src/service.rs`) — the typed contract:
  - `trait Service { type Request: Message; type Response: Message; }`
  - `trait ServiceImpl { type Service; async fn handle_request(&self, req) -> resp; }` (server side; must be `Clone`).
  - `trait ServiceCaller<S> { async fn call(&self, req: S::Request) -> Result<S::Response, ClientError>; }` (client side).
  - `fn service_caller<S: Service>(client: Arc<Client>) -> Box<dyn ServiceCaller<S>>`.
  - `ServiceId` is `std::any::type_name::<S>()` — services are keyed by Rust type name.
- `ConnectionAddress` (`src/protocol.rs`) — newtype over a socket path; `ConnectionAddress::new()` mints `/tmp/warp-ipc-{rand}.sock`. `From<String>`/`Display` for passing it to a child process via env var.

### Internal modules

- `protocol.rs` — wire format: length-prefixed (`USIZE_SIZE`) `bincode`-serialized `Request`/`Response` frames; `Message` blanket trait (`Send + Sync + 'static + Clone + Serialize + DeserializeOwned`).
- `platform` — selected by `cfg`: `native.rs` (interprocess + `async-compat`) vs `wasm.rs` (planned MessagePort, unimplemented). The `#[cfg_attr(..., path = ...)]` trick maps both to `mod platform`.
- `testing.rs` — in-process test harness (`#[cfg(test)] pub mod testing`).

## Depends on (internal)

- [`warpui_core`](./warpui_core.md) — for the async executor abstraction. Uses `warpui_core::r#async::executor::{Background, BackgroundTask}` to spawn the per-connection reader/writer loops, and `warpui_core::r#async::block_on(...)` to bind the listener (`native.rs`). This is the crate's only internal dependency.

## Used by (internal dependents)

- [`warp`](./warp.md) (the `app` crate) — the entire `app/src/plugin/` subsystem. Notable call sites:
  - `app/src/plugin/app/mod.rs` — builds the app-side `ipc::Server` (`ServerBuilder::default()...`) and holds an optional `host_client: Arc<ipc::Client>`; exposes `plugin_service_caller<S: ipc::Service>()`.
  - `app/src/plugin/host/native/service_impl.rs` — `impl ipc::ServiceImpl for CallJsFunctionServiceImpl`.
  - `app/src/plugin/host/native/mod.rs` — child plugin-host process reads `ipc::ConnectionAddress` from the `PLUGIN_HOST_ADDRESS_ENV_VAR` and connects back.

## Related crates

- [`warpui_core`](./warpui_core.md) — the executor it builds on.
- [`command-signatures-v2`](./command-signatures-v2.md) — the built-in completions plugin loaded into the very plugin host this IPC layer serves.
- The `local_control` crate (Warp's *other* IPC channel, used by `warp_cli` for app control) — conceptually a sibling: both are local cross-process protocols, but `local_control` is for CLI→running-app control, while `ipc` is for app↔plugin-host.

## Marley relevance

**Classify: KEEP (rename later) — foundational, not on the critical path.**

- This is generic, well-isolated plumbing with **one internal dep and one dependent**, so it is cheap to keep as-is and carries no Warp-specific business logic — none of the four goals require changing it.
- **De-Warp rebrand (goal 4):** only cosmetic — `ConnectionAddress::new()` hardcodes the socket prefix `"/tmp/warp-ipc-"`. Rename to `/tmp/marley-ipc-` so two installs don't collide and to avoid the Warp string. Trivial one-line change.
- **Session spawn/read/write (goal 2):** the module docs explicitly flag this protocol as extensible "to other use cases (such as the terminal server)." If Marley grows its own terminal-server or panel-host process, this is the ready-made typed transport to reuse rather than reinventing.
- **STUB note:** if Marley ships without the JS plugin system initially, the `ipc` server in `app/src/plugin/app/mod.rs` can be left un-instantiated (it is already `Option<ipc::Server>`) — the crate compiles and links with zero behavior, no stubbing inside `ipc` itself required.
- No de-auth relevance (goal 3): this layer has no notion of credentials or login.

## Notes / gotchas

- **Out-of-repo dep:** `interprocess = "1.2.1"` (native only, with `tokio_support`); pulled in under `cfg(not(target_family = "wasm"))`.
- **WASM is a stub:** `wasm.rs` exists but the MessagePort transport is unimplemented — building for `wasm32-unknown-unknown` will not give you working IPC.
- **`ServiceId` = type name:** services are identified by `std::any::type_name`, so renaming/moving a `Service` type changes its on-wire identity — client and server must be built from the same type definitions (fine since both live in this monorepo).
- **`async-compat`** bridges Tokio (interprocess) into the `futures` `AsyncRead`/`AsyncWrite` world used by `protocol.rs`; the listener bind is wrapped in `.compat()` and `block_on`.
- Wire framing is hand-rolled length-prefix + `bincode`; not self-describing or versioned — both ends must agree on `bincode` encoding.
