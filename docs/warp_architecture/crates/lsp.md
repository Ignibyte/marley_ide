# lsp

> Per-crate reference (Marley round 2) — crate dir `crates/lsp`. Marley is forked from Warp ([warpdotdev/warp](https://github.com/warpdotdev/warp)).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` for Warp's client code, but the capability is a `[permissive: public LSP spec + MIT lsp-types + in-repo jsonrpc]` surface any reimplementation builds from clean. Marley has **no editor LSP yet** (deferred with the editor subsystem) — no counterpart today. See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).

| Field | Value |
|-------|-------|
| Subsystem | [07 — App Entry & Build Tooling](../subsystems/07-app-entry-build-tooling.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`); no own LICENSE marker. |
| Internal deps | **9** |
| Used by | **1** |

## Purpose

`lsp` is Warp's **Language Server Protocol client** — a stdio-only LSP transport plus a manager that
spawns, installs, and routes requests to language servers (rust-analyzer, gopls, pyright,
typescript-language-server, clangd). Per its `README.md`: it spawns and manages a language-server
child process and communicates over stdio using JSON-RPC with Content-Length framing. It exists so
the editor/code surfaces in the `warp` app can offer hovers, diagnostics, references, and completions
backed by real language servers, and it knows how to auto-install missing servers via a node runtime.

## Key types, modules & public API

Re-exported from `crates/lsp/src/lib.rs`:

- **`pub async fn spawn_lsp_service(config, executor, logger) -> Result<LspServiceInitializationResult>`**
  — the main entry: resolves `config.command_and_params()`, builds a `transport::ProcessTransport`,
  wraps it in a `jsonrpc::JsonRpcService`, constructs and `initialize`s an `LspService`, and returns
  the service plus a `Receiver<ServerNotificationEvent>`. There is a WASM stub that returns
  "LSP is not supported in WASM environments."
- **`pub fn init(app: &mut AppContext)`** — registers the singleton `LspManagerModel` in the UI app.
- **`pub struct LspService`** (`service.rs`) and **`LspServiceInitializationResult { service, channel }`**.
- **`LspManagerModel` / `LspManagerModelEvent`** (`manager.rs`) — the per-workspace registry:
  `servers_for_workspace`, `server_for_path`, `register`, `start_all`/`stop_all`, `remove_server`,
  `terminate`, `maybe_register_external_file`, `workspace_roots`.
- **`config::{LspServerConfig, LanguageId, default_init_params}`** — server configuration and the
  resolved-command path.
- **`model::{LspServerModel, LspState, LspEvent, LanguageServerId, DocumentDiagnostics, BackgroundTaskInfo}`**
  — per-server state/events.
- **`supported_servers::{LSPServerType, CustomBinaryConfig}`** — the enum of known servers with
  `binary_name()`, `languages()`, `language_name()`, `candidate(client)`, and `all()`.
- **`types::{HoverContents, HoverResult, MarkupKind, ReferenceLocation}`**, plus re-exports of
  `lsp_types::{Position, Range, notification}` and `jsonrpc::{JsonRpcService, Transport, ServerNotificationEvent}`.
- `pub mod install` — language-server installation; `pub mod servers` — per-server adapters;
  `LanguageServerCandidate` (`language_server_candidate.rs`) — a discovered/installable server.
- Transport: `transport::ProcessTransport` (`transport.rs`, non-wasm) and the wasm/non-wasm split of
  `server_repo_watcher`.

## Depends on (internal)

- [`jsonrpc`](./jsonrpc.md) — the JSON-RPC engine (`JsonRpcService`, `Transport`,
  `ServerNotificationEvent`) that `LspService` is built on.
- [`node_runtime`](./node_runtime.md) — runs/installs node-based servers (pyright, typescript).
- [`command`](./command.md) — process spawning for the language-server child (`ProcessTransport`).
- [`http_client`](./http_client.md) — downloading server binaries; `candidate(client: Arc<http_client::Client>)`.
- [`repo_metadata`](./repo_metadata.md) — repo/workspace root detection for per-workspace routing.
- [`warp_core`](./warp_core.md) — core app types/channel/feature glue.
- [`warp_util`](./warp_util.md) — shared utilities.
- [`warpui_core`](./warpui_core.md) — `AppContext` / async `Background` executor and the singleton
  model registration in `init`.
- [`simple_logger`](./simple_logger.md) — the optional `SimpleLogger` that captures LSP stderr to a file.

## Used by (internal dependents)

- [`warp`](./warp.md) — the only dependent; the app calls `lsp::init(app)` and `spawn_lsp_service`
  from its editor/code surfaces.

## Related crates

- [`jsonrpc`](./jsonrpc.md) — sibling transport layer; `lsp` is essentially a JSON-RPC client with
  LSP semantics.
- [`languages`](./languages.md) / [`syntax_tree`](./syntax_tree.md) — language identification and
  parsing the editor uses alongside LSP.
- [`node_runtime`](./node_runtime.md) — the runtime that hosts JS/TS-based servers.

## Marley relevance

**Classify: KEEP (low priority).** This is feature plumbing with **no Warp branding and no
cloud-auth coupling** — it talks to local language-server processes over stdio, not to Warp's
backend. None of the four Marley goals require touching it:
- It is orthogonal to de-auth (goal 3) and rebrand (goal 4).
- It is not on the core session spawn/write/read path (goal 2) — that's `terminal`/`command`.
- A custom panel (goal 1) *could* consume it (e.g. surface diagnostics), but that's additive.

So **keep as-is**. The only branding nit is the package name `lsp` is already generic (no rename
needed) and the `dev@warp.dev` author metadata, which is cosmetic. If Marley ships a slimmer offline
build, this crate can even be feature-gated off without affecting boot.

## Notes / gotchas

- **stdio-only.** No TCP/socket transport — only `ProcessTransport` spawning a child over
  stdin/stdout with Content-Length framing.
- **WASM is unsupported at runtime:** `spawn_lsp_service` is a hard `Err(...)` on `wasm32`, and
  `server_repo_watcher` swaps to `server_repo_watcher_wasm.rs`. `transport` is cfg'd out on wasm.
- **Out-of-repo dep:** `lsp-types = "0.97.0"` (crates.io), re-exported as the public `Position`/`Range`/
  `notification` surface — upgrades to that crate are API-visible.
- **Auto-install** of servers (`install` module + `node_runtime`) reaches the network/filesystem;
  relevant if Marley wants a fully-offline mode.
- `local_fs` feature exists; `examples/rust-lsp/main.rs` is a runnable reference client.
