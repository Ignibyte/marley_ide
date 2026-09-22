# Marley Crate Reference

> **Marley** is Ignibyte's fork of the open-source Warp terminal
> ([warpdotdev/warp](https://github.com/warpdotdev/warp)). This directory is the
> **round-2** documentation layer: **one reference doc per workspace crate** (77
> crates). It is the granular companion to the subsystem surveys in
> [`../subsystems/`](../subsystems/) and the last doc-gathering pass before
> implementation begins. **Still documentation only — no source was modified.**

## How this fits together

| Layer | Where | Granularity |
|---|---|---|
| Project map | [`../subsystems/00-overview.md`](../subsystems/00-overview.md) | The corpus overview + doc index |
| Subsystem surveys | [`../subsystems/00`–`07`](../subsystems/) | One doc per subsystem (data flow, seams, hook points) |
| **Crate reference (this dir)** | `./<pkg>.md` | **One doc per crate** — `Cargo.toml`, `lib.rs`/`main.rs`, key modules, real `pub` types/fns, deps/dependents, Marley classification |

Each crate doc cites its exact internal edges from
[`../.depgraph.json`](../.depgraph.json) (the **ground-truth** dependency graph,
keyed by package name with `dir` / `deps` / `dependents`). Counts below come from
that file.

**Marley classification key:** **KEEP** (carry as-is) · **EXTEND** (build on it)
· **RENAME** (de-Warp branding, deferred unless noted cheap) · **STUB** (replace
behavior with a local/no-op seam — usually the de-auth/cloud cut) · **REMOVE**
(drop from the Marley build).

---

## License map

**MIT:** [`warpui`](./warpui.md), [`warpui_core`](./warpui_core.md),
[`warpui_extras`](./warpui_extras.md) — the GUI framework + platform backend.
**Every other crate is AGPL v3.** Per-crate docs note any crate that carries its
own `LICENSE` marker.

---

## Foundational crates (highest blast radius)

These are the most depended-on hubs in the graph — any rename, STUB, or signature
change here ripples across the whole workspace. Touch them last and deliberately.

| Crate | Dependents | Role |
|---|---:|---|
| [`warpui_core`](./warpui_core.md) | **34** | MIT retained-mode GUI framework (`App`/entities/`Element`/`Presenter`/`Scene`) — the app's foundation |
| [`warp_core`](./warp_core.md) | **33** | App/runtime core: `SessionId`, channel branding + server URLs, paths, feature flags, telemetry |
| [`command`](./command.md) | **17** | Drop-in `std`/`async` process `Command` wrapper (forces Windows `CREATE_NO_WINDOW`); substrate for PTY spawn |
| [`warp_util`](./warp_util.md) | **16** | Shared-utility grab-bag (`FileId`, `ContentVersion`, `StandardizedPath`, git/path/sync helpers) |
| [`string-offset`](./string-offset.md) | **12** | Type-safe `CharOffset`/`ByteOffset` newtypes — the shared text-position vocabulary |

Runners-up worth caution: [`warp_graphql`](./warp_graphql.md) (9, highest cloud
fan-out), [`warpui`](./warpui.md) (8), [`settings_value`](./settings_value.md) (7),
[`warpui_extras`](./warpui_extras.md) (7). Note: [`warp`](./warp.md) has the most
*outgoing* deps (65) — it is the apex consumer, not a hub.

---

## Crates by subsystem

### UI framework & rendering — [`../subsystems/01`](../subsystems/01-ui-framework-rendering.md)

- [`warpui_core`](./warpui_core.md) — MIT retained-mode GUI framework (`App`/entities/handles/`Element`/`Presenter`/`Scene`); 34 dependents. **[KEEP, rename deferred]**
- [`warpui`](./warpui.md) — MIT platform & rendering backend (winit/wgpu/Metal/wasm) that re-exports `warpui_core` and rasterizes a `Scene`. **[KEEP, rename deferred]**
- [`warpui_extras`](./warpui_extras.md) — `secure_storage` (Keychain/Secret-Service/DPAPI) + `user_preferences` models registered into `AppContext`; the de-auth hinge. **[STUB `secure_storage` / KEEP `user_preferences`]**
- [`ui_components`](./ui_components.md) — reusable widget library (button/switch/dialog/tooltip/lightbox) on the Component/Params/Options pattern. **[KEEP + EXTEND]**
- [`asset_cache`](./asset_cache.md) — URL- and `data:`-URI-backed async `AssetSource` layer over `warpui_core`'s `AssetCache`, with hashed-file persistence. **[KEEP]**
- [`asset_macro`](./asset_macro.md) — proc-macro resolving compile-time bundled/remote/split asset refs into `AssetSource` (verifies existence, SHA-256-hashes remote). **[KEEP]**
- [`warp_assets`](./warp_assets.md) — rust-embed runtime asset embedder; `struct Assets` impls `warpui_core::AssetProvider`, baking `app/assets` into the binary. **[KEEP]**
- [`watcher`](./watcher.md) — debounced cross-platform FS watcher as a `warpui_core` `Entity` emitting add/modify/delete/move diffs (`BulkFilesystemWatcher`/`HomeDirectoryWatcher`). **[KEEP]**

### Editor & text — [`../subsystems/02`](../subsystems/02-editor-and-text.md)

- [`warp_editor`](./warp_editor.md) — rope-backed text-editing core: buffer/selection/anchor/undo/find + layout & render pipeline. **[KEEP / EXTEND]**
- [`sum_tree`](./sum_tree.md) — dependency-free copy-on-write summarizing B-tree (the rope engine). **[KEEP]**
- [`string-offset`](./string-offset.md) — type-safe `CharOffset`/`ByteOffset` newtypes, shared text-position vocabulary; 12 dependents. **[KEEP]**
- [`syntax_tree`](./syntax_tree.md) — incremental tree-sitter highlight/auto-indent runtime (`SyntaxTreeState`). **[KEEP]**
- [`languages`](./languages.md) — embedded tree-sitter grammar/query registry, 34 languages. **[KEEP]**
- [`markdown_parser`](./markdown_parser.md) — parses markdown/HTML into Warp's diffable `FormattedText` line/inline model (tables, task lists, action-hyperlinks); 0 internal deps, 5 dependents. **[KEEP]**
- [`fuzzy_match`](./fuzzy_match.md) — Skim-based fuzzy scoring + regex-free glob/wildcard path matching with highlight indices. **[KEEP]**
- [`vim`](./vim.md) — modal-editing FSA (`VimFSA`/`VimModel`) emitting `VimEvent`s via the `VimHandler` seam, plus generic word/paragraph/bracket/text-object motions. **[KEEP]**
- [`ipynb_parser`](./ipynb_parser.md) — render-only nbformat-v4 `.ipynb` → `FormattedText` converter (markdown/code/output cells) with raw-text fallback; used by the editor's `Buffer`. **[KEEP]**

### Terminal & session core — [`../subsystems/03`](../subsystems/03-terminal-session-core.md)

- [`warp_terminal`](./warp_terminal.md) — UI-agnostic terminal model primitives (grid, `BlockId`/`BlockIndex`, ANSI/escape, shell detection), adapted from Alacritty; no PTY. **[KEEP]**
- [`warp_core`](./warp_core.md) — app/runtime core: `SessionId`, `ChannelState`/`WarpServerConfig` branding + server URLs, paths, feature flags, telemetry; 33 dependents. **[KEEP / EXTEND, rename deferred]**
- [`warp_tui`](./warp_tui.md) — headless ratatui front-end (`TuiInputView`) + per-channel bins calling `warp::run_tui()`; leaf, 0 dependents. **[KEEP / EXTEND, cheap RENAME]**
- [`local_control`](./local_control.md) — `warpctrl` external-automation protocol: `ActionKind` catalog (tab/session/input), envelopes, auth, discovery; no raw-byte/output action. **[EXTEND + de-auth STUB]**
- [`command`](./command.md) — drop-in `std`/`async` process `Command` wrapper forcing Windows `CREATE_NO_WINDOW`; substrate for `spawn_command_in_pty`; 17 dependents. **[KEEP]**
- [`command-signatures-v2`](./command-signatures-v2.md) — rust-embed shim baking a compiled TS/JS `Completions` command-signatures bundle into the binary. **[KEEP (drop Node build step) / STUB if completions cut]**
- [`ipc`](./ipc.md) — generic typed request/response IPC framework (interprocess UDS/named-pipes) for the app↔plugin-host channel. **[KEEP (rename socket prefix; reuse for future session transport)]**
- [`warp_cli`](./warp_cli.md) — clap argument-parsing + dispatch for the multi-personality Warp/oz binary (GUI, workers, CLI, warpctrl control). **[EXTEND / RENAME (de-auth login stub, rebrand, session control)]**
- [`warp_completer`](./warp_completer.md) — the completions engine: parses the command line, emits ranked fuzzy suggestions, exposes `ParsedTokensSnapshot` to the classifier. **[KEEP, RENAME deferred, STUB v2 JS engine]**

### Agent, AI & MCP — [`../subsystems/04`](../subsystems/04-agent-ai-mcp.md)

- [`ai`](./ai.md) — client-side domain model for Agent Mode: agent action/result type system, API-key/credential management, codebase index + embeddings, skills, project context. **[KEEP / EXTEND, rename deferred]**
- [`mcp`](./mcp.md) — Model Context Protocol client runtime (rmcp facade): spawns stdio/HTTP/SSE servers, lists tools/resources, OAuth, returns `TemplatableMCPServerInfo`. **[KEEP]**
- [`warp_multi_agent_client`](./warp_multi_agent_client.md) — network transport for Agent Mode: authenticates and streams the cloud multi-agent SSE response, decoding base64+protobuf `ResponseEvent`s. **[STUB, rename deferred]**
- [`computer_use`](./computer_use.md) — OS-level GUI control (mouse/keyboard/screenshot) across mac/win/X11/Wayland; fulfils the agent `UseComputer` action; ships `use_computer` CLI. **[KEEP, optionally STUB to no-op]**
- [`jsonrpc`](./jsonrpc.md) — small transport-agnostic JSON-RPC 2.0 client (`JsonRpcService` + `Transport` trait) used by the LSP subsystem. **[KEEP]**
- [`natural_language_detection`](./natural_language_detection.md) — leaf lexical scorer (dictionaries + stemmer) rating how natural-language-like input is, feeding the AI-vs-shell decision. **[KEEP]**
- [`input_classifier`](./input_classifier.md) — decides Shell vs AI for the unified prompt via a BERT-tiny ONNX model + heuristic + completer signal, with graceful fallback. **[KEEP, STUB the ONNX path for offline boot]**
- [`lsp`](./lsp.md) — stdio-only Language Server Protocol client + per-workspace `LspManagerModel`; spawns/installs rust-analyzer/gopls/pyright/tsserver/clangd over JSON-RPC. **[KEEP]**

### Cloud, auth & networking — [`../subsystems/05`](../subsystems/05-cloud-auth-networking.md)

- [`warp_server_auth`](./warp_server_auth.md) — client-side auth state (who's logged in, credentials, anonymous id, secure-storage persistence) as the `AuthStateProvider` singleton; central de-auth lever via `skip_login`/`Credentials::Test`. **[KEEP + STUB]**
- [`warp_server_client`](./warp_server_client.md) — action layer performing network login/refresh, OAuth2 device flow, IAP, API keys, user-settings sync, Warp Drive access via the `AuthClient` trait; primary network short-circuit point. **[KEEP + STUB]**
- [`firebase`](./firebase.md) — serde-only models for Google Firebase/Identity Platform REST responses (`AccountInfo`, token/error envelopes); no logic or networking. **[STUB / REMOVE-candidate]**
- [`warp_graphql`](./warp_graphql.md) — typed cynic GraphQL client + operations (37 queries / 71 mutations / subscriptions), transport, scalars, IAP/staging error mapping; highest cloud fan-out (9 dependents). **[KEEP, partial STUB]**
- [`warp_graphql_schema`](./warp_graphql_schema.md) — single registration of the `warp-server` GraphQL SDL via cynic codegen; leaf plumbing crate. **[KEEP, rename deferred]**
- [`cloud_objects`](./cloud_objects.md) — model-agnostic cloud-object substrate: ids (`ServerId`/`SyncId`/`ClientId`), `ObjectType`, metadata/permissions, `drive::sharing`. **[KEEP]**
- [`cloud_object_models`](./cloud_object_models.md) — concrete typed Warp Drive models (workflow/notebook/folder/mcp/env_vars/cloud_environment) + `Cloud*`/`Server*` aliases. **[KEEP / EXTEND]**
- [`cloud_object_persistence`](./cloud_object_persistence.md) — shared model-agnostic SQLite plumbing: callback upsert/delete, permission encoding, refresh scheduling. **[KEEP / EXTEND]**
- [`cloud_object_client`](./cloud_object_client.md) — the async `ObjectClient` trait (~35 methods) abstracting all Warp Drive CRUD/sync/sharing; mockable seam, no networking itself. **[STUB]**
- [`onboarding`](./onboarding.md) — first-run multi-slide welcome flow + post-onboarding callouts; presents login/skip and Warp branding. **[STUB / RENAME]**
- [`warp_managed_secrets`](./warp_managed_secrets.md) — client-side managed-secrets library: typed secret values, HPKE/tink envelope encryption, GraphQL-backed manager, GCP workload-identity federation. **[STUB]**
- [`managed_secrets_wasm`](./managed_secrets_wasm.md) — wasm-bindgen shim exposing `warp_managed_secrets` envelope encryption (Anthropic/OpenAI/Bedrock keys) to the browser dashboard. **[REMOVE]**
- [`websocket`](./websocket.md) — single native+wasm WebSocket abstraction (tungstenite/rustls vs ws_stream_wasm) implementing `graphql_ws_client` transport, with proxy + IAP-challenge handling. **[KEEP]**
- [`http_client`](./http_client.md) — shared reqwest wrapper for all networked subsystems: identity headers, request/response hooks, IAP token provider, SSE eventsource, prevent-sleep; native+wasm. **[KEEP]**
- [`http_server`](./http_server.md) — embedded loopback axum server (127.0.0.1, per-channel port from `PORT_BASE` 9277) hosting app-contributed routers, mainly OAuth redirect callbacks. **[STUB]**
- [`warp_web_event_bus`](./warp_web_event_bus.md) — WASM-only Rust→JS event shim (`WarpEvent` enum + `emit_event`) for the browser build to notify its JS host. **[STUB / RENAME]**
- [`remote_server`](./remote_server.md) — app-side SSH remote-dev client: `RemoteServerManager`/`RemoteServerClient` driving a prebuilt remote daemon over length-prefixed protobuf for remote files/git/search/codebase-index. **[KEEP (defer) + STUB auth]**
- [`serve-wasm`](./serve-wasm.md) — standalone dev webserver (axum/clap) serving the prebuilt Warp-on-Web wasm bundle + assets on 127.0.0.1. **[REMOVE / keep-as-dev-tool]**
- [`warp_isolation_platform`](./warp_isolation_platform.md) — detects Docker/DockerSandbox/Kubernetes/Namespace sandboxes and issues workload-identity tokens for Warp-hosted agents. **[STUB]**
- [`field_mask`](./field_mask.md) — protobuf `google.protobuf.FieldMask` partial-update merge (update/append) via prost-reflect `DynamicMessage`. **[KEEP (dormant once cloud-sync stubbed)]**

### Platform, settings, persistence & infra — [`../subsystems/06`](../subsystems/06-platform-settings-infra.md)

- [`settings`](./settings.md) — typed declarative settings framework (`Setting` trait + `define_settings_group!` macros + `SettingsManager` registry) routing values across TOML / native store / secure storage / cloud sync. **[KEEP]**
- [`settings_value`](./settings_value.md) — defines the `SettingsValue` trait controlling the TOML-settings-file representation (parallel to serde); tiny leaf with broad reuse (7 dependents). **[KEEP]**
- [`settings_value_derive`](./settings_value_derive.md) — proc-macro `#[derive(SettingsValue)]` generating snake_case/recursive file-serialization impls. **[KEEP]**
- [`persistence`](./persistence.md) — SQLite/Diesel local durable-state layer (50 tables: windows/tabs/panes/blocks/workflows/agent convos) + embedded migrations. **[KEEP]**
- [`virtual-fs`](./virtual-fs.md) — test-fixture filesystem builder (`VirtualFS`/`Stub`/`Dirs` + Warp path helpers); dev-only, not a runtime VFS. **[KEEP / RENAME helper]**
- [`prevent_sleep`](./prevent_sleep.md) — cross-platform RAII guard (macOS objc2 / Windows / no-op) keeping the OS awake during long work, plus a `Stream` wrapper. **[KEEP]**
- [`node_runtime`](./node_runtime.md) — downloads a pinned Node.js (v22.12.0) into the data dir or validates system Node, plus npm-registry helpers, for the `lsp` crate. **[KEEP]**
- [`warp_files`](./warp_files.md) — central `FileModel` for async open/read/watch/save of local & remote files, emitting `FileModelEvent`, plus a line-range text reader. **[KEEP / EXTEND]**
- [`warp_util`](./warp_util.md) — workspace grab-bag of shared utilities (`FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, asset-dir constants, git, path, sync, worktree names); 16 dependents. **[KEEP, rename deferred]**
- [`warp_logging`](./warp_logging.md) — logging init/rotation/panic-hooks/crash-bundle (`LogConfig` + `init`), native file vs wasm browser, optional Sentry `crash_reporting` feature. **[KEEP]**
- [`simple_logger`](./simple_logger.md) — async file-based sink for subprocess (MCP/LSP) stderr with size-rotation and a singleton `LogManager`. **[KEEP]**
- [`channel_versions`](./channel_versions.md) — data model + parsing/override logic for Warp's dev/preview/stable update-channel version metadata. **[STUB]**
- [`warp_channel_config`](./warp_channel_config.md) — loads per-channel `ChannelConfig` (embed-at-build vs runtime `warp-channel-config` generator) via the `load_config!` macro. **[STUB]**
- [`warp_features`](./warp_features.md) — app-wide `FeatureFlag` registry backed by lock-free atomic arrays; zero internal deps, transitively ubiquitous via `warp_core`. **[KEEP / EXTEND]**
- [`app-installation-detection`](./app-installation-detection.md) — tiny axum loopback endpoint (`/install_detection`) letting warp.dev detect an installed app via hardcoded warp.dev CORS. **[REMOVE]**
- [`repo_metadata`](./repo_metadata.md) — reactive in-app model of git repos + file trees (detect/index/watch, local & remote); source for path autocomplete, file panels, AI context, standing queries. **[KEEP]**
- [`handlebars`](./handlebars.md) — homegrown `{{name}}` template substitution (NOT crates.io handlebars), Unicode-correct; `get_arguments` + `render_template`. **[KEEP / RENAME low-pri (name collision)]**
- [`warp_js`](./warp_js.md) — Rust↔JS plugin bridge over embedded QuickJS (rquickjs): typed function registry, refs, cross-process bincode marshalling, `FromWarpJs`/`IntoWarpJs`. **[KEEP, rename deferred]**
- [`warp_ripgrep`](./warp_ripgrep.md) — thin ripgrep wrapper: in-process parallel JSON search worker + async/streaming client that re-execs the host binary via a `warp_cli` subcommand. **[KEEP]**
- [`warp_search_core`](./warp_search_core.md) — Tantivy-backed full-text search + a generic result-mixing/ranking framework (`SearchMixer`, data sources, `QueryResultRenderer`) powering the command-palette/fuzzy-finder menus. **[KEEP / EXTEND]**
- [`voice_input`](./voice_input.md) — capture-only mic recorder that resamples to 16kHz mono and emits a base64 WAV for external (Wispr) transcription; single `VoiceInput` entity. **[STUB]**

### App entry, build & tooling — [`../subsystems/07`](../subsystems/07-app-entry-build-tooling.md)

- [`warp`](./warp.md) — the application crate (`app/`): apex of the graph (65 deps), multi-bin/single-lib GUI terminal; owns root view, auth UI, terminal/session engines, channel bins + `Info.plist`. **[EXTEND + rename deferred]**
- [`integration`](./integration.md) — end-to-end test harness: `Builder`/`TestStep` scenario scripting + `warp-integration-test` binary that boots the real app headless against black-holed servers. **[KEEP, EXTEND later]**

---

## Complete file list

All 77 per-crate docs in this directory (every workspace crate has exactly one):

[`ai.md`](./ai.md) ·
[`app-installation-detection.md`](./app-installation-detection.md) ·
[`asset_cache.md`](./asset_cache.md) ·
[`asset_macro.md`](./asset_macro.md) ·
[`channel_versions.md`](./channel_versions.md) ·
[`cloud_object_client.md`](./cloud_object_client.md) ·
[`cloud_object_models.md`](./cloud_object_models.md) ·
[`cloud_object_persistence.md`](./cloud_object_persistence.md) ·
[`cloud_objects.md`](./cloud_objects.md) ·
[`command-signatures-v2.md`](./command-signatures-v2.md) ·
[`command.md`](./command.md) ·
[`computer_use.md`](./computer_use.md) ·
[`field_mask.md`](./field_mask.md) ·
[`firebase.md`](./firebase.md) ·
[`fuzzy_match.md`](./fuzzy_match.md) ·
[`handlebars.md`](./handlebars.md) ·
[`http_client.md`](./http_client.md) ·
[`http_server.md`](./http_server.md) ·
[`input_classifier.md`](./input_classifier.md) ·
[`integration.md`](./integration.md) ·
[`ipc.md`](./ipc.md) ·
[`ipynb_parser.md`](./ipynb_parser.md) ·
[`jsonrpc.md`](./jsonrpc.md) ·
[`languages.md`](./languages.md) ·
[`local_control.md`](./local_control.md) ·
[`lsp.md`](./lsp.md) ·
[`managed_secrets_wasm.md`](./managed_secrets_wasm.md) ·
[`markdown_parser.md`](./markdown_parser.md) ·
[`mcp.md`](./mcp.md) ·
[`natural_language_detection.md`](./natural_language_detection.md) ·
[`node_runtime.md`](./node_runtime.md) ·
[`onboarding.md`](./onboarding.md) ·
[`persistence.md`](./persistence.md) ·
[`prevent_sleep.md`](./prevent_sleep.md) ·
[`remote_server.md`](./remote_server.md) ·
[`repo_metadata.md`](./repo_metadata.md) ·
[`serve-wasm.md`](./serve-wasm.md) ·
[`settings.md`](./settings.md) ·
[`settings_value.md`](./settings_value.md) ·
[`settings_value_derive.md`](./settings_value_derive.md) ·
[`simple_logger.md`](./simple_logger.md) ·
[`string-offset.md`](./string-offset.md) ·
[`sum_tree.md`](./sum_tree.md) ·
[`syntax_tree.md`](./syntax_tree.md) ·
[`ui_components.md`](./ui_components.md) ·
[`vim.md`](./vim.md) ·
[`virtual-fs.md`](./virtual-fs.md) ·
[`voice_input.md`](./voice_input.md) ·
[`warp.md`](./warp.md) ·
[`warp_assets.md`](./warp_assets.md) ·
[`warp_channel_config.md`](./warp_channel_config.md) ·
[`warp_cli.md`](./warp_cli.md) ·
[`warp_completer.md`](./warp_completer.md) ·
[`warp_core.md`](./warp_core.md) ·
[`warp_editor.md`](./warp_editor.md) ·
[`warp_features.md`](./warp_features.md) ·
[`warp_files.md`](./warp_files.md) ·
[`warp_graphql.md`](./warp_graphql.md) ·
[`warp_graphql_schema.md`](./warp_graphql_schema.md) ·
[`warp_isolation_platform.md`](./warp_isolation_platform.md) ·
[`warp_js.md`](./warp_js.md) ·
[`warp_logging.md`](./warp_logging.md) ·
[`warp_managed_secrets.md`](./warp_managed_secrets.md) ·
[`warp_multi_agent_client.md`](./warp_multi_agent_client.md) ·
[`warp_ripgrep.md`](./warp_ripgrep.md) ·
[`warp_search_core.md`](./warp_search_core.md) ·
[`warp_server_auth.md`](./warp_server_auth.md) ·
[`warp_server_client.md`](./warp_server_client.md) ·
[`warp_terminal.md`](./warp_terminal.md) ·
[`warp_tui.md`](./warp_tui.md) ·
[`warp_util.md`](./warp_util.md) ·
[`warp_web_event_bus.md`](./warp_web_event_bus.md) ·
[`warpui.md`](./warpui.md) ·
[`warpui_core.md`](./warpui_core.md) ·
[`warpui_extras.md`](./warpui_extras.md) ·
[`watcher.md`](./watcher.md) ·
[`websocket.md`](./websocket.md)
