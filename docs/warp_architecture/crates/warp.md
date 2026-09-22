# warp

> Per-crate reference (Marley round 2) — crate dir `app/`. Marley is forked from Warp ([warpdotdev/warp](https://github.com/warpdotdev/warp)).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` — Warp's apex app crate (the `License` row below is Warp's, not Marley's). Marley did **not** fork it: the counterpart is `crates/marley_app` (`[Marley-original]` — a clean-room ~48-line `run()` boot + gpui `RootView`, MIT-OR-Apache-2.0). Warp's UI framework `warpui` is `[permissive: MIT]`; Marley builds on `[permissive: gpui Apache-2.0]`. See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).

| Field | Value |
|-------|-------|
| Subsystem | [07 — App Entry & Build Tooling](../subsystems/07-app-entry-build-tooling.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`); no own LICENSE marker. |
| Internal deps | **65** |
| Used by | **2** |

## Purpose

`warp` is **the application crate** — the top of the dependency graph and the thing that becomes the
shipped terminal binary. It lives in `app/` (the package is named `warp`, with `[lib] name = "warp"`).
It wires together essentially every other workspace crate (AI, terminal, editor, settings, auth, LSP,
MCP, cloud objects, voice, search, …) into one running GUI app, and it owns all of the product-level
UI: the root window, workspace/pane layout, command palette, settings views, auth modals, AI
assistant panels, onboarding, themes, and so on.

It is also a **multi-headed binary**: one library, many `[[bin]]` targets that differ only in
`Channel` identity (icon, app name, server config, feature flags). The bins live in `app/src/bin/`:
`oss.rs` (`warp-oss`, the OSS build), `local.rs` (`warp`, dev), `dev.rs`, `preview.rs`, `stable.rs`,
plus `integration.rs` (test runner entry) and the `generate_settings_schema.rs` tool. Each bin
constructs a `warp_core::channel::ChannelState`, calls `ChannelState::set(...)`, then calls
`warp::run()`.

## Key types, modules & public API

Entry points (`app/src/lib.rs`):

- **`pub fn run() -> Result<()>`** — the real entry point every channel bin calls. It does
  `platform::init()`, `features::init_feature_flags()`, parses `warp_cli::Args::from_env()`, dispatches
  worker subcommands (terminal server, plugin host, minidump server, remote-server proxy/daemon,
  ripgrep search subprocess), CLI/SDK commands (`warp_cli::Command::CommandLine`), `Completions`, and
  `DumpDebugInfo` — and finally launches the GUI via `run_internal(LaunchMode::App { … })`.
- **`pub fn run_integration_test(driver: TestDriver) -> Result<()>`** — entry used by the `integration`
  crate's runner; wraps `run_internal(LaunchMode::Test { … })`.
- **`pub fn run_tui() -> Result<()>`** (feature `tui`) — headless TUI front-end used by `warp_tui`.
- **`fn run_internal(launch_mode: LaunchMode)`** — the shared bootstrap; `pub enum LaunchMode`
  (`App`, `CommandLine`, `Test`, `RemoteServerProxy`, `RemoteServerDaemon`, `Tui`) selects behavior.

Notable **public** modules (the crate deliberately keeps most modules private — see the big
"PLEASE DO NOT ADD MORE PUBLIC MODULES" banner in `lib.rs`): `pub mod channel`, `features`,
`appearance`, `editor`, `keyboard`, `root_view`, `search`, `settings`, `settings_view`, `terminal`,
`themes`, `launch_configs`, `resource_center`, `ai_assistant`, `input_suggestions`, `pane_group`,
`tab_configs`, and (feature-gated) `integration_testing` (which exposes `view_getters` /
`assertions` so tests don't need internal types).

Important internal modules a Marley dev will touch:

- `app/src/root_view.rs` — `pub struct RootView`, `RootView::new(...)`, window/quake-mode management
  (`QuakeModeState`, `create_transferred_window`), and the many `open_*_in_existing_window` handlers.
- `app/src/auth/` — `auth_manager.rs` (`pub struct AuthManager`, `AuthManagerEvent`, sign-in/up URL
  builders, `create_anonymous_user`, `initialize_user_from_session_cookie`), plus all the login UI
  (`auth_view_modal.rs`, `login_slide.rs`, `paste_auth_token_modal.rs`, `web_handoff.rs`,
  `needs_sso_link_view.rs`).
- `app/src/session_management.rs` — `SessionNavigationData` and session navigation/sharing state.
- `app/src/server/`, `app/src/remote_server/`, `app/src/terminal/` — local PTY + remote session engines.
- `app/build.rs` — asset hashing/copying and `cfg_aliases!` (`linux_or_windows`, `enable_crash_recovery`).

## Depends on (internal)

65 internal crates — effectively the whole workspace. The structurally important ones:

- [`warp_core`](./warp_core.md) — `Channel`/`ChannelState`/`ChannelConfig`/`AppId`/`features`; the bins
  and `run()` are built on it.
- [`warpui`](./warpui.md), [`warpui_core`](./warpui_core.md), [`warpui_extras`](./warpui_extras.md) —
  the GUI framework (`AppContext`, `Entity`, views) the whole app is built in (MIT).
- [`warp_cli`](./warp_cli.md) — `Args`, `Command`, `WorkerCommand`, `AppArgs`; all argument parsing
  and CLI/worker dispatch.
- [`warp_channel_config`](./warp_channel_config.md) — `load_config!` macro that bakes per-channel config.
- [`warp_terminal`](./warp_terminal.md), [`command`](./command.md), [`warp_editor`](./warp_editor.md),
  [`vim`](./vim.md) — terminal/PTY, process spawning, editor surface.
- [`ai`](./ai.md), [`mcp`](./mcp.md), [`computer_use`](./computer_use.md),
  [`warp_multi_agent_client`](./warp_multi_agent_client.md), [`input_classifier`](./input_classifier.md) —
  the agentic / AI assistant stack.
- [`lsp`](./lsp.md) — language-server client used by the editor/code surfaces.
- [`warp_server_auth`](./warp_server_auth.md), [`warp_server_client`](./warp_server_client.md),
  [`firebase`](./firebase.md), [`warp_graphql`](./warp_graphql.md),
  [`warp_managed_secrets`](./warp_managed_secrets.md) — auth + backend/cloud transport.
- [`cloud_objects`](./cloud_objects.md), [`cloud_object_client`](./cloud_object_client.md),
  [`cloud_object_models`](./cloud_object_models.md), [`cloud_object_persistence`](./cloud_object_persistence.md),
  [`persistence`](./persistence.md), [`settings`](./settings.md), [`settings_value`](./settings_value.md) —
  state, settings, Warp Drive objects.
- [`remote_server`](./remote_server.md), [`local_control`](./local_control.md), [`ipc`](./ipc.md),
  [`http_server`](./http_server.md), [`http_client`](./http_client.md), [`websocket`](./websocket.md),
  [`watcher`](./watcher.md), [`virtual-fs`](./virtual-fs.md) — transport/IO plumbing.
- Plus: [`onboarding`](./onboarding.md), [`voice_input`](./voice_input.md), [`vim`](./vim.md),
  [`languages`](./languages.md), [`syntax_tree`](./syntax_tree.md), [`markdown_parser`](./markdown_parser.md),
  [`fuzzy_match`](./fuzzy_match.md), [`warp_ripgrep`](./warp_ripgrep.md),
  [`warp_search_core`](./warp_search_core.md), [`warp_completer`](./warp_completer.md),
  [`warp_files`](./warp_files.md), [`warp_isolation_platform`](./warp_isolation_platform.md),
  [`warp_js`](./warp_js.md), [`warp_logging`](./warp_logging.md), [`warp_util`](./warp_util.md),
  [`warp_assets`](./warp_assets.md), [`warp_web_event_bus`](./warp_web_event_bus.md),
  [`warp_managed_secrets`](./warp_managed_secrets.md), [`asset_cache`](./asset_cache.md),
  [`asset_macro`](./asset_macro.md), [`handlebars`](./handlebars.md), [`simple_logger`](./simple_logger.md),
  [`string-offset`](./string-offset.md), [`sum_tree`](./sum_tree.md), [`field_mask`](./field_mask.md),
  [`channel_versions`](./channel_versions.md), [`command-signatures-v2`](./command-signatures-v2.md),
  [`app-installation-detection`](./app-installation-detection.md), [`repo_metadata`](./repo_metadata.md),
  [`natural_language_detection`](./natural_language_detection.md), [`ui_components`](./ui_components.md),
  [`warp_managed_secrets`](./warp_managed_secrets.md).

## Used by (internal dependents)

Only **2** (it sits at the apex of the graph):

- [`integration`](./integration.md) — the integration-test runner depends on `warp` (with the
  `integration_tests` feature) and drives the real app headlessly via `run_integration_test`.
- [`warp_tui`](./warp_tui.md) — the headless TUI binary, which boots the app through `run_tui`.

## Related crates

- [`warp_core`](./warp_core.md) — owns `Channel`/`ChannelState`; read it alongside the bin entry points.
- [`warp_cli`](./warp_cli.md) — the arg/command surface `run()` dispatches on; the de-auth and
  login-stub work touches both crates.
- [`warp_channel_config`](./warp_channel_config.md) — the per-channel config that the bins bake in.
- [`integration`](./integration.md) and [`warp_tui`](./warp_tui.md) — the two non-GUI front-ends.

## Marley relevance

**Classify: EXTEND + RENAME (rename-deferred).** This is the crate Marley actually ships and the
center of gravity for all four goals:

1. **Custom panel (goal 1):** new product UI lands here — add a module under `app/src/` and wire it
   into `root_view.rs` (`RootView` / `open_*_in_existing_window`) and the command palette
   (`app/src/command_palette.rs`). This is the primary `EXTEND` surface.
2. **Session spawn/write/read (goal 2):** the PTY/session engines live under `app/src/terminal/`,
   `app/src/server/`, `app/src/remote_server/`, and `app/src/session_management.rs`; any
   programmatic session API Marley exposes is built/threaded here.
3. **De-auth + login stub (goal 3):** `app/src/auth/` is the concrete target — short-circuit
   `AuthManager` (e.g. always `create_anonymous_user` / synthesize a logged-in user, skip the
   Firebase/`warp_server_auth` round-trips) and suppress the login modals (`auth_view_modal.rs`,
   `login_slide.rs`). Pair with the same change in `warp_server_auth`/`firebase`.
4. **De-Warp rebrand (goal 4):** the channel bins (`app/src/bin/*.rs`) carry the **`Info.plist`**
   (`CFBundleDisplayName`, `CFBundleIdentifier = dev.warp.*`, URL schemes `warposs`/`warplocal`,
   copyright "Denver Technologies, Inc"), `AppId::new("dev","warp", …)`, log file names, and channel
   names — all of which must be rebranded. App assets/icons live under `app/assets/` and
   `app/resources/`.

Renaming the **package** `warp` → `marley` (and `[lib] name`) is desirable for the rebrand but it is
the apex crate that 2 others import by name and that the build/bundling tooling targets by binary
name (`warp-oss`, `warp`); rename is mechanical but high-blast-radius, so **defer the package rename
until the channel/branding pass** and do the user-visible rebrand (plist/AppId/strings/icons) first.

## Notes / gotchas

- **Do not add public modules.** `lib.rs` has an explicit banner: public modules lose dead-code
  analysis; expose test hooks via `integration_testing::assertions` / `view_getters` instead.
- **Multi-bin, single-lib.** `default-run = "warp-oss"`, `autobins = false`; every channel bin is
  "exactly identical except channel config." Branding diffs live in the bins, not the lib.
- **Embedded `Info.plist`** via `embed_plist::embed_info_plist_bytes!` unless the `extern_plist`
  feature is set — rebrand must edit these inline plist strings (or switch to extern plist).
- **Heavily feature-gated.** The `[features]` table is huge (`default` enables `agent_mode`,
  `viewing_shared_sessions`, `minimalist_ui`, … dozens). `crash_reporting`, `plugin_host`,
  `local_tty`, `tui`, `local_fs`, `standalone` gate large code paths. Build matrix matters.
- `build.rs` hashes/copies assets and depends on `warp_util::assets` constants and `app_target_dir`;
  it also defines `cfg_aliases!` (`linux_or_windows`, `enable_crash_recovery`).
- Worker subcommands re-enter the **same binary** (terminal server, remote-server daemon/proxy,
  minidump server, ripgrep) instead of separate binaries — see the match in `run()`.
