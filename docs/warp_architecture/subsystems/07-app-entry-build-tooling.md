# Subsystem 07 — App Entry, Build & Tooling

Part of the Marley architecture docs — **round 3 re-review** (originally round 1). This doc is a
**behavior reference for Warp's** app-entry/build source (`warpdotdev/warp`, AGPL-3.0); Marley
reimplements the capability clean-room (CONSTITUTION §20) rather than forking `app/`. Read
**[Marley status @ M15](#marley-status--m15)** and **[Provenance & licensing](#provenance--licensing)**
below before the Warp analysis in §1–§9.

> Scope: the top-level `app/` crate (binary entry points, the `warp::run()` boot
> sequence, where the UI tree and panels are assembled and registered), the
> `script/` developer tooling, the `lsp`/`integration`/`warp_cli` supporting
> crates, and the workspace-level build/toolchain configuration
> (`Cargo.toml`, `rust-toolchain.toml`, `flake.nix`, `docker/`).
>
> **Round-3 correction:** the round-1 premise below — "Marley will *hook into* this `app/` crate to
> add a panel and stub the login" — was **superseded**. Marley did not fork `app/`; it built its own
> clean-room app shell (`crates/marley_app`). §1–§9 stand as the **Warp behavior reference** — the
> hook-point analysis and the inline "> Marley … note" callouts describe *Warp's* seams, not Marley's
> shipped design. **Read-only doc — no Warp source was modified or vendored.**

---

## Marley status @ M15

Marley has **its own app-entry, its own build/gate tooling, and its own pipeline** — none forked from
Warp. This subsystem is therefore ~85% reference (understand Warp's boot/branding/channel shape) and
~15% relevant (the boot *pattern* and the Workspace→panes *layout* that Marley reimplemented clean).

**Marley's app-entry (the analog of `app/` + `warp::run()`)** — `crates/marley_app`
(`[package] marley`, `[lib] marley_app`, `[[bin]] marley`; doc:
[`marley_architecture/app_shell.md`](../../marley_architecture/app_shell.md)):

- `src/bin/marley.rs` → `fn main() -> ExitCode { marley_app::run() }` — one bin, no channel matrix.
- `marley_app::run()` (`src/app.rs`) ≈ **48 lines**: `Application::new().with_assets(Assets).run(|cx| …)`
  opens **one** gpui window (saved-geometry-or-centred 1024×768, unified transparent titlebar) rooted in
  `RootView::new(window, cx)`, then `cx.activate(true)`. Contrast Warp's ~2850-line `lib.rs` `run()` →
  `run_internal` → `AppBuilder::run` with CLI/worker dispatch, single-instance forwarding, crash
  reporting, and platform menu wiring (§2). Marley's boot has **almost none** of that — the one
  exception, since **#380**, is a minimal application menu: `run()` declares a `Quit` gpui action,
  binds `cmd-q`, and `set_menus` a "Marley" menu with a single "Quit Marley" item (the standard macOS
  ⌘Q accelerator, teardown-parity with the AppleEvent quit via `[NSApp terminate:]`). Still no
  CLI/worker dispatch, single-instance forwarding, or crash reporting.
- **No `initialize_app`-style flat `Module::init(ctx)` registry.** Marley wires its regions in
  `RootView::new` + the pure seams `layout` (PaneGroup algebra + dock states), `workspace`/`tabs`
  (Workspace→Project→Tab→PaneGroup + focus model), `palette` (fuzzy command list), `keymap` (chord →
  action). Everything is a unit-tested **pure** module; only `app.rs` is the accepted-untestable gpui
  **shim** (coverage-excluded + `mutants::skip`), asserted by the headed harness.

**Which Warp app-entry pieces are RELEVANT (as behavior reference):** the boot *shape* (single entry →
build a root-view-owning window → run the platform event loop); the three-region **Workspace** layout
(§2's left/center/right — Marley reimplements as docks + a center `PaneGroup` in `layout.rs`/
`workspace.rs`/`tabs.rs`); and the single-window `RootView` container concept (§2). The `integration`
crate (§5) is the reference for a headless GUI-drive harness.

**Which are MARLEY-SPECIFIC / already built (not adopted):**

- **Build & gate tooling** — `scripts/gates.sh` (the **15-gate** quality bar: fmt/clippy/tests/coverage/
  mutation/miri/audit/deny/machete/gitleaks/shellcheck/no-suppress/SAST/docs/visual-AX; cov + MSI floors
  = 100) is Marley's `script/presubmit` (§6) analog, but strict-from-day-one with no baselines. The
  headed **gate-15** (gpui + AXUIElement + screenshot) drives `crates/marley_visual_harness` — Marley's
  `integration`-crate analog.
- **The pipeline** — CONSTITUTION **§0–§21**: `/work` pre-flight → `/pipeline:plan → design → implement
  → inspect (§18.1) → validate → complete (§21 docs) → /commit`, enforced by `.claude/hooks/enforce-*.sh`
  PreToolUse gates. This is the entire Marley delivery model; Warp's `script/` runner (§6) has no equivalent.
- **Toolchain** — Marley is a plain Cargo workspace (`resolver = "2"`, `members = ["crates/*"]`); it
  vendors nothing and has no `flake.nix`/`docker/`/channel-config generator (§7).

**Which Warp pieces are NOT relevant (removed / never built in Marley):** channel binaries + embedded
`Info.plist` + `AppId`/URL-schemes/copyright (§2) — Marley ships one un-channelled `marley`; the
CLI/worker multi-personality (`warp_cli`, §5) — single binary, no `oz`/terminal-server re-exec;
`onboarding` — replaced by the **M13 workspace launcher** (`src/launcher.rs`, the "open a workspace"
landing page); `channel_versions`/`warp_channel_config` — no update channels or auto-update;
`app-installation-detection` — no website→desktop bridge; `serve-wasm` — desktop-gpui only, no
wasm/web build; `auth::init` (§4) — no mandatory login to stub in the first place.

## Provenance & licensing

This subsystem's doc + its crate docs **describe Warp's AGPL source as a behavior reference**; Marley's
app-entry is largely **Marley-original already** — written clean-room from behavior specs, never
translated from Warp source (§20). Tags:

| Tag | What it covers here |
|-----|---------------------|
| **`[Warp-derived/AGPL]`** | Everything the §1–§9 body and the crate docs *analyze*: Warp's `app`/`warp::run()`, channel bins + `Info.plist`, `warp_cli` dispatch, `onboarding`, `channel_versions`/`warp_channel_config`, `app-installation-detection`, `serve-wasm`, `integration`, `lsp`. **Warp's AGPL-3.0 source is never committed or vendored into this repo** — these are Marley's own descriptions. (The crate docs' `License: AGPL v3` field row describes *Warp's* workspace license, not Marley's.) |
| **`[permissive]`** | The frameworks/libs Warp's app-entry builds on that carry **no copyleft** and Marley may reuse freely (gate:8 allowlist): Warp's UI framework `warpui` (**MIT**) ↔ Marley uses **gpui (Apache-2.0)**; `axum`/`clap` (serve-wasm, warp_cli); the public **LSP wire spec + MIT `lsp-types`** + the in-repo `jsonrpc` (lsp). |
| **`[Marley-original: the boot, the gate/pipeline tooling]`** | `crates/marley_app` — the `run()` boot, the gpui `RootView`, and the pure `layout`/`workspace`/`tabs`/`palette`/`keymap` seams; `crates/marley_visual_harness`; `scripts/gates.sh`; the CONSTITUTION §0–§21 pipeline; `.claude/hooks/`. **No Warp lineage** — the highest-confidence clean-room surface in the whole product. |

**Licensing posture (verify/refresh).** Marley's workspace currently declares
`license = "MIT OR Apache-2.0"` (root `Cargo.toml [workspace.package]`) — a **permissive placeholder for
its clean-room code, not AGPL**. The intended *outcome* (chad, open-core): GPL/AGPL the editor+terminal
layer and keep the **brain** proprietary and clean of any Warp/Zed-derived code; the app-entry + gate/
pipeline tooling is Marley-original and can sit on either side of that line. **IP-counsel sign-off is
pending** (`docs/marley_architecture/clean-build-plan.md`, §20) — settle it before commercializing.
This is why app-entry/build is the *lowest-risk* subsystem: there is almost nothing Warp-derived to
relicense — the boot and the tooling were original from day one.

---

## 1. Workspace shape

`Cargo.toml` (repo root) declares a Cargo workspace, `resolver = "2"`, members
`crates/*` + `app`. License is `AGPL-3.0-only` (`[workspace.package]`), and
`publish = false`. There are **78 crates** under `crates/` plus the `app` crate.

`default-members` is intentionally a *subset* (just `app` + a handful of leaf
crates like `warpui`, `warp_terminal`, `warp_completer`, `command`, `editor`,
`graphql`, `sum_tree`, …). The comment explains the omissions: `serve-wasm` is a
wasm-serving helper, and `integration` is test-only. So a bare `cargo build`
builds the app and its transitive deps, not the whole tree.

All internal crates are referenced through `[workspace.dependencies]` by path
(e.g. `warp = { path = "app" }`, `warpui = { path = "crates/warpui" }`,
`warp_core = { path = "crates/warp_core" }`). The MIT-licensed UI framework
crates called out in the Marley charter are here: `warpui`, `warpui_core`,
`warpui_extras` (the last with `default-features = false`).

Toolchain & build config:

- `rust-toolchain.toml` → `channel = "1.92.0"`, components `rustfmt`+`clippy`,
  `profile = "minimal"`. Pins the exact compiler.
- `.cargo/config.toml` → sets `MACOSX_DEPLOYMENT_TARGET = "10.14"`,
  `rustflags = ["-C","symbol-mangling-version=v0", "-C","link-args=-Wl,-headerpad_max_install_names"]`
  (the headerpad flag leaves room for code-signing/`install_name` rewrites in the
  macOS bundle step), `git-fetch-with-cli = true`, and a wasm `web_sys_unstable_apis`
  cfg for the wasm target.
- `.rustfmt.toml`, `.clippy.toml`, `deny.toml` (cargo-deny), `about.toml`/`about.hbs`
  (cargo-about license report), `diesel.toml` (DB schema) round out tooling.

---

## 2. The `app` crate — binaries and entry points

`app/Cargo.toml`: `name = "warp"`, `default-run = "warp-oss"`, `autobins = false`
(bins are declared explicitly). The lib target is `name = "warp"`, `path = "src/lib.rs"`.

The crate compiles to **one of several near-identical channel binaries** — each is
a thin `fn main()` wrapper that configures a `ChannelState` and then calls the
shared `warp::run()`:

| `[[bin]]` | path | channel | notes |
|-----------|------|---------|-------|
| `warp-oss` | `src/bin/oss.rs` | `Channel::Oss` | **default-run**; the open-source build. Embeds its own `Info.plist` (bundle id `dev.warp.WarpOss`, URL scheme `warposs`). |
| `warp` | `src/bin/local.rs` | `Channel::Local` | internal dev build; loads `warp_channel_config::load_config!("local")`, layers `DEBUG/DOGFOOD/PREVIEW/LOCAL` feature flags. |
| `dev` | `src/bin/dev.rs` | `Channel::Dev` | `load_config!("dev")` + DEBUG/DOGFOOD/PREVIEW flags. |
| `stable` | `src/bin/stable.rs` | `Channel::Stable` | release channel. |
| `preview` | `src/bin/preview.rs` | `Channel::Preview` | gated behind `required-features = ["preview_channel"]`. |
| `integration` | `src/bin/integration.rs` | — | integration-test harness binary. |
| `generate_settings_schema` | `src/bin/generate_settings_schema.rs` | — | codegen helper (also `examples/generate_default_settings.rs`). |

Each channel binary differs only in: `AppId`, log-file name, server/telemetry/
crash/autoupdate config, and the embedded `Info.plist` (display name + bundle id +
URL scheme + `NSHumanReadableCopyright "© 2026, Denver Technologies, Inc"`). The
plist is embedded via `embed_plist::embed_info_plist_bytes!` unless the
`extern_plist` feature is on.

> **Rebrand note:** the per-binary `Info.plist` strings, `AppId::new("dev","warp","WarpOss")`,
> the `warposs`/`warplocal` URL schemes, and the `Denver Technologies` copyright are
> the literal user-visible branding. They live in the `src/bin/*.rs` wrappers (not
> buried in the framework), so a Marley rebrand is mostly a controlled edit of these
> small files + asset swaps, leaving `warp::run()` untouched.

### Boot sequence: `warp::run()` → `run_internal()` → `AppBuilder::run()`

`app/src/lib.rs` (~2850 lines) is the heart of the app. The module list at the
top is mostly **private** modules, with an explicit comment *"PLEASE DO NOT ADD
MORE PUBLIC MODULES"* — public modules lose dead-code analysis, so the crate
exposes only a small public surface (`channel`, `features`, `editor`,
`ai_assistant`, `appearance`, `input_suggestions`, `root_view`, `util`,
`integration_testing`, …).

`pub fn run() -> Result<()>` (lib.rs:631) is the shared entry:

1. `platform::init()` and `features::init_feature_flags()`.
2. **CLI dispatch** — checks `warp_cli::local_control::ControlArgs` (control-mode
   env) and parses `warp_cli::Args::from_env()`. If the invocation is a worker
   subcommand or CLI command, it never starts the GUI:
   - `WorkerCommand::TerminalServer` → `terminal::local_tty::server::run_terminal_server`
     (the single-binary "terminal server" re-exec, behind `local_tty`).
   - `WorkerCommand::PluginHost` → `run_plugin_host()` (behind `plugin_host`).
   - `WorkerCommand::MinidumpServer`, `RemoteServerProxy`, `RemoteServerDaemon`,
     `RipgrepSearch` → their respective workers.
   - `Command::Completions` / `CommandLine` (the `oz` agent CLI) / `DumpDebugInfo`
     / `PrintTelemetryEvents` → handled then return.
   - If invoked as a CLI binary (`standalone` feature, binary name starts `oz`, or
     `WARP_CLI_MODE` set) it just prints help.
3. Otherwise it calls `run_internal(LaunchMode::App { args, api_key })`.

`fn run_internal(launch_mode: LaunchMode)` (lib.rs:822) does the heavy lifting:
- Pre-app init: profiling, feature flags, crash reporting (Sentry), logging
  (`warp_logging::init`), `resource_limits::adjust_resource_limits()`, single-instance
  forwarding (`app_services::{linux,windows}::pass_startup_args_to_existing_instance`),
  Windows job-object setup, and settings preload
  (`settings::init_private_user_preferences` / `init_public_user_preferences`).
- Creates the PTY spawner (`terminal::local_tty::spawner::PtySpawner::new()`, behind
  `local_tty`) — skipped for the TUI launch mode to avoid a fork bomb.
- Builds `warpui::platform::AppBuilder` (`AppBuilder::new` or `::new_headless`),
  passing `AppCallbacks`, the embedded `ASSETS`, and an optional test driver.
  Platform-specific configuration is applied here: macOS menu bar
  (`app_builder.set_menu_bar_builder(app_menus::menu_bar)`), dock menu, dev icon,
  dock-icon visibility; Linux X11/Wayland + window class; Windows DXC shader path
  and app-user-model-id. Custom keybinding triggers are converted/registered.
- `app_builder.run(move |ctx| { … })` — this closure runs **inside the platform
  event loop** with an `&mut warpui::AppContext`. It registers singleton models
  (`AppExecutionMode`, the `PtySpawner`, `PublicPreferences`/`PrivatePreferences`,
  optional `PluginHost`/`CrashRecovery`), then calls `initialize_app(...)`, then —
  for the GUI/CLI path — `launch(ctx, app_state, launch_mode)`. (For `--features tui`
  and `LaunchMode::Tui` it calls `crate::tui::init(ctx)` and returns instead.)

`warpui::platform::AppBuilder` itself lives in `crates/warpui/src/platform/app.rs`
(`pub struct AppBuilder`, `::new`, `::new_headless`, `pub fn run(self, init_fn)`),
i.e. the windowing/event-loop framework is in the MIT `warpui` crate, driven from
`app/`.

`LaunchMode` (enum in lib.rs) is the single switch distinguishing `App`,
`CommandLine` (oz CLI), `Tui`, `Test`, and `RemoteServerProxy` modes; helpers like
`needs_profiling`, `needs_crash_reporting`, `is_headless`, `log_destination`,
`execution_mode` drive the branches above.

### `initialize_app()` — **the panel/feature registration site**

`pub(crate) fn initialize_app(launch_mode, timer, …, ctx: &mut AppContext, …)`
(lib.rs:1190) is where the entire app's views, models, and **panels** are wired
into the `AppContext`. The pattern is a long, flat sequence of `Module::init(ctx)`
calls plus `ctx.add_singleton_model(...)`. The relevant block (lib.rs ≈1782–1840):

```
ai::init(ctx);
app_services::init(ctx);
code::editor::find::view::init(ctx);
workspace::init(ctx);          // top-level Workspace view
pane_group::init(ctx);         // terminal pane grid
terminal::init(ctx);           // terminal panes / PTY views
input::init(ctx);
editor::init(ctx);
onboarding::init(ctx);
menu::init(ctx);
…
root_view::init(ctx);          // the per-window RootView
voltron::init(ctx);
auth::init(ctx);               // ← login/auth wiring  (app/src/auth/mod.rs:56)
…
ai_assistant::panel::init(ctx);  // ← the AI assistant panel
ai::agent::todos::popup::init(ctx);
coding_entrypoints::project_buttons::init(ctx);
if FeatureFlag::CodeReviewSaveChanges.is_enabled() { code_review::init(ctx); }
… many ctx.add_singleton_model(...) calls (DisplayCount, ToastStack, VimRegisters,
   FileSearchModel, SystemStats, NetworkStatus, KeybindingChangedNotifier, …) …
```

**This `*::init(ctx)` list is the canonical place Marley adds a new panel.** A new
Marley panel module would expose its own `pub fn init(app: &mut AppContext)`
(registering bindings/actions and any singleton model) and get one new line added
here, exactly like `ai_assistant::panel::init(ctx)`.

### UI tree assembly: Workspace → left/center/right panels

The visible window UI is the `Workspace` view (`app/src/workspace/view.rs`,
~28.6k lines — the largest file in the app). Its struct holds the three-pane
layout (view.rs ≈1130–1135):

```
left_panel_open: bool,
left_panel_view:  ViewHandle<LeftPanelView>,    // app/src/workspace/view/left_panel.rs
left_panel_views: Vec<ToolPanelView>,
right_panel_view: ViewHandle<RightPanelView>,   // app/src/workspace/view/right_panel.rs
… plus the center PaneGroup of terminal panes …
```

- **Center** = the terminal grid: `crate::pane_group::PaneGroup` / `PaneId` /
  `PanesLayout` with pane kinds including terminal panes, `FilePane`,
  `NetworkLogPane`, `EnvironmentManagementPane`, `ExecutionProfileEditorPane`,
  `CustomRouterEditorPane`, etc. (imported in view.rs from `crate::pane_group`).
  `terminal::init(ctx)` + `pane_group::init(ctx)` register these.
- **Left** = `LeftPanelView` (project explorer / global search / Warp Drive /
  **agent conversations** — see binding names `workspace:left_panel_*` around
  view.rs:643). Built at view.rs:3006 via `LeftPanelView::new(...)`.
- **Right** = `RightPanelView` (`app/src/workspace/view/right_panel.rs`, ~1900
  lines). This is the **Agent Mode / code-review side panel**. Its struct
  (right_panel.rs:423) holds `active_pane_group: Option<ViewHandle<PaneGroup>>`,
  `is_agent_management_view_open: bool`, `code_review_state`, `panel_position`,
  resize state, etc. It is constructed at view.rs:3018
  (`RightPanelView::new(working_directories_model, ctx)`) and the Workspace
  subscribes to its `RightPanelEvent`s (view.rs:3021). Toggle binding:
  `workspace:toggle_right_panel` (`TOGGLE_RIGHT_PANEL_BINDING_NAME`, view.rs:629).

So "the Agent Mode panel" surfaces as: (a) the `RightPanelView` (agent management
+ review side panel), (b) agent conversations in the `LeftPanelView`, and (c) the
AI assistant panel registered by `ai_assistant::panel::init(ctx)`. The per-window
container is `RootView` (`app/src/root_view.rs`, ~3580 lines; `pub fn init(app)`
at root_view.rs:258 registers its bindings, global window shortcuts, and the
`root_view:open_new` / quake-mode / notification actions). `RootView` owns the
`Workspace` and tab/`ActiveSession` machinery (the `Workspace`, `ActiveSession`,
`Toast­Stack`, `PaneViewLocator`, `WorkspaceAction` types are re-exported from
`crate::root_view`, lib.rs:283–320).

> **Marley UI-surface note:** to add a panel that visualizes an agentic workflow,
> the cleanest seam is a new view module with `pub fn init(ctx)` added to the
> `initialize_app` list, then either (a) a new `ToolPanelView` variant in the
> `LeftPanelView`/`RightPanelView` set, or (b) a new pane kind in
> `crate::pane_group`. The right panel (`RightPanelView`) is the closest existing
> analog to "a panel that hosts agent sessions/state" and is the lowest-risk place
> to model Marley's custom panel after.

---

## 3. Session spawn / write / read — where it bottoms out

The terminal-session machinery is split between `app/src/terminal/` (views) and the
single-binary **terminal server**:

- `PtySpawner` (`terminal::local_tty::spawner`) is created once in `run_internal`
  and registered as a singleton; it spawns PTYs (it is the thing
  `run_internal` is careful to set up "in the cleanest possible process state").
- The terminal server is **not a separate binary** — `warp::run()` re-execs the
  same binary with `WorkerCommand::TerminalServer`, which lands in
  `terminal::local_tty::server::run_terminal_server(args)` and never returns to
  the GUI. The code comments make this explicit ("it's much easier to distribute a
  single binary, so starting the terminal server event loop immediately is the
  closest approximation we can get to running a separate binary").
- Programmatic session spawn/write/read **already exists** via the `oz` agent CLI:
  `warp_cli::Command::CommandLine(CliCommand::Agent(AgentCommand::Run(...)))` →
  `run_internal(LaunchMode::CommandLine { … })`. `warp_cli/src/agent.rs` plus
  `local_control` (the `ControlArgs::from_control_mode_env()` path at the top of
  `run()`) are how an external process drives a Warp session today.

> **Marley session spawn/write/read note:** Marley's "spawn a terminal session,
> write to it, read output back" requirement maps onto (1) the `PtySpawner`
> singleton + `terminal::local_tty` server for the actual PTY, and (2) the
> `warp_cli` `local_control` / `oz agent run` control surface for the
> programmatic API. A Marley panel that drives sessions should reuse the
> `pane_group`/terminal views for the UI and the existing local-control/agent
> plumbing for the spawn/write/read, rather than inventing a new PTY path.

---

## 4. De-auth seam

Login wiring enters at `auth::init(ctx)` in the `initialize_app` list
(`app/src/auth/mod.rs:56`, `pub fn init(app: &mut AppContext)`), backed by the
`warp_server_auth` crate (`crates/warp_server_auth/src/lib.rs`): `AuthStateProvider`,
`auth_state`, `credentials`, `user`, `user_uid::UserUid`, `anonymous_id`
(API-key prefix `wk-`). The `app/Cargo.toml` default feature set already includes
`loginless_conversion` (feature defined at Cargo.toml:866) and
`external_agent_mode_context`, signaling Warp already supports a partially-anonymous
mode.

> **Marley de-auth note:** the single `auth::init(ctx)` call is the choke point.
> Stubbing it (or making it return a fixed anonymous `AuthStateProvider` /
> `UserUid`) is how Marley removes the mandatory login while leaving a clean seam
> for Ignibyte's future login. The `anonymous_id` + `loginless_conversion`
> machinery in `warp_server_auth` is the existing scaffolding to build that stub
> on. Server-URL overrides are deliberately ignored on release channels
> (`Channel::allows_server_url_overrides()`, enforced in `run()`), so any
> Marley-hosted backend redirect must change channel config, not just env vars.

---

## 5. Supporting crates surveyed

### `crates/warp_cli` (`name = "warp_cli"`, `edition = 2024`)
Pure CLI-argument + subcommand layer (clap derive). `lib.rs` defines `Args`,
`Command`, `WorkerCommand`, `CliCommand`, `binary_name()`, env-var constants
(`OZ_RUN_ID_ENV`, `WARP_SERVER_ROOT_URL` override, …). Submodules: `agent`
(the `oz agent run` surface + `OutputFormat`/`Harness`), `api_key`, `completions`
(`clap_complete`), `config_file`, `local_control` (control-mode bridge),
`mcp`, `model`, `provider`, `schedule`, `secret`, `share`, `task`, `skill`,
`artifact`, `scope`, `federate`. Depends on `warp_core`, `warp_util`,
`local_control`. **This crate is the programmatic front door** — it parses what
becomes the worker/CLI/agent branches in `warp::run()`.

### `crates/lsp` (`name = "lsp"`)
Language Server Protocol client used by the editor/code features. `lib.rs`
re-exports `LspManagerModel`/`LspManagerModelEvent`, `LspServerModel`, `LspService`,
`LspState`, `CommandBuilder`, `LanguageServerCandidate`, `install`,
`supported_servers`. Built on the in-repo `jsonrpc` crate (`JsonRpcService`,
`Transport`) + `node_runtime` + `lsp-types 0.97`. Has a `local_fs` feature and a
wasm-vs-native `server_repo_watcher` split. Peripheral to Marley's core goals but
part of the app's IDE surface.

### `crates/integration` (`name = "integration"`)
Test-only crate (`[[bin]] integration`, `test = false`; excluded from
`default-members`). `lib.rs` re-exports `Builder`, `test::*`, `user_defaults`,
`util`, plus `warp::integration_testing::view_getters` and
`warpui_core::integration::TestStep` — i.e. it drives the app through the same
`integration_testing` assertion API the lib comment described. This is the harness
Marley would extend to assert against a new panel without making app internals public.

---

## 6. `script/` — developer & build tooling

`script/` is a bash-based task runner (≈40 entries). Key ones:

| script | role |
|--------|------|
| `bootstrap` | one-time checkout setup; dispatches to platform `install_*` scripts (`install_rust`, `install_cargo_build_deps`, `install_cargo_bundle`, …) and installs common agent skills from `skills-lock.json`. Each root step sources `warp_sudo` and prompts (`-y`/`WARP_SKIP_SUDO_PROMPT=1` to skip). |
| `run` | cross-platform **build + run**. Defaults `FEATURES="gui"`. Runs `install_channel_config`, then if `warp-channel-config` is on PATH builds the **Local** channel (`warp` bin), else the **OSS** channel (`warp-oss`). On macOS delegates to `script/macos/run` (real `.app` bundle + signing + plist), on Linux/Windows runs `cargo run` directly. Maps legacy `--features` to env vars; `--` passes args through to the binary. |
| `run-tui` | runs the `warp_tui` crate's `warp-tui` (Local) / `warp-tui-oss` (OSS) console binary via `cargo run -p warp_tui`. |
| `presubmit` | the local CI gate (67 lines): `script/format` (rustfmt), `check_no_inline_test_modules`, `cargo clippy --workspace --exclude warp_completer --all-targets --tests -- -D warnings` (+ a separate `-p warp_completer` clippy run with default features), `run-clang-format.py` over `crates/warpui/src` + `app/src`, `wgslfmt --check` on `*.wgsl` shaders, optional PowerShell lint, then **`cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`** (+ `warp_completer --features v2`) and `cargo test --doc`. |
| `bundle` / `macos/*` / `linux/*` / `windows/*` | packaging: `.app`/`.dmg`, Linux, Windows bundles; `update_plist`, `compile_icon`, `Entitlements.plist`, font patching (`patch_font_with_warp_glyph`, `font_fallback`). |
| `install_channel_config` | fetches the internal `warp_channel_config` (gated by repo access — OSS contributors fall back to the `oss` channel). |
| `wasm`, `serve-wasm` helper, `deploy_remote_server*`, `create_release_tag_and_branch`, `sentry_*` | wasm build, remote-server deploy, release & symbol upload. |
| `copy_conditional_skills` / `resolve_common_skills` | agent-skill plumbing (skills are a first-class concept in this repo: `.agents/`, `skills-lock.json`, `.warp/`). |

> **Marley build note:** the practical local loop is `./script/bootstrap` once,
> then `./script/run` (which yields `warp-oss` for an OSS checkout) and
> `./script/presubmit` as the merge gate. The `gui` Cargo feature is the default
> entry feature; `agent_mode` is in the default feature set (`app/Cargo.toml`
> default = […, "agent_mode", "agent_mode_primary_xml", …]). A Marley fork without
> internal repo access naturally builds the **OSS channel**, which is exactly the
> branding surface to rename.

---

## 7. Toolchain reproducibility: `flake.nix` + `docker/`

- `flake.nix` — *experimental, Linux-only* Nix flake
  (`x86_64-linux`, `aarch64-linux`). Uses `crane` + `rust-overlay`, deriving the
  Rust toolchain from `rust-toolchain.toml` (`fromRustupToolchainFile`). It vendors
  Cargo deps "Zed-style" from `Cargo.lock` (no top-level vendor hash) and patches
  build scripts for `warp_multi_agent_api` protos and `warp-workflows` specs (the
  `specs/` dir, 231 entries, is copied to `nix-vendored-specs`). Version is
  `app/Cargo.toml`'s version + `self.shortRev`. Declares Linux runtime libs
  (alsa, etc.). `flake.lock` pins inputs.
- `docker/agent-dev/Dockerfile` and `docker/linux-dev/Dockerfile` (+ README) —
  containerized dev/agent environments. `.dockerignore` present.
- `rust-toolchain.toml` is the single source of truth for the compiler version,
  consumed by both Nix and plain rustup checkouts.

---

## 8. Dependencies on other subsystems

`app` sits at the top of the dependency graph and pulls in essentially everything:
- **UI framework**: `warpui` (windowing/`AppBuilder`/`AppContext`/views),
  `warpui_core`, `warpui_extras`, `ui_components`, `editor`.
- **Core domain**: `warp_core` (channels/features/`AppId`), `warp_terminal`,
  `command`, `pane_group` (in-crate), `settings`/`settings_value`.
- **CLI / control**: `warp_cli`, `local_control`, `ipc`, `jsonrpc`, `mcp`.
- **AI/agents**: `ai`, `warp_multi_agent_client`, `computer_use`, `voltron` (in-crate).
- **Auth/server**: `warp_server_auth`, `warp_server_client`, `firebase`, `graphql`.
- **Infra**: `warp_logging`, `simple_logger`, `persistence`, `virtual_fs`, `watcher`,
  `http_client`/`http_server`, `websocket`, `lsp`, `node_runtime`, `warp_ripgrep`.

The inversion to remember: **`app/` owns the boot sequence and the registration
list, but the windowing/event loop and view primitives live in the MIT `warpui`
crates.** Marley's panel work is mostly additive code in `app/` (a new module +
one `init(ctx)` line + a panel/pane-kind variant), using `warpui` view APIs.

---

## 9. Marley relevance summary

| Marley goal | Where it lands in this subsystem |
|-------------|----------------------------------|
| **UI-surface expansion / new panel** | Add `Module::init(ctx)` to the `initialize_app` list (`app/src/lib.rs` ≈1782–1840) and a new `ToolPanelView`/pane variant; model it on `RightPanelView` (`app/src/workspace/view/right_panel.rs`) or `LeftPanelView`. Per-window container is `RootView` (`app/src/root_view.rs`). |
| **Session spawn / write / read** | `PtySpawner` singleton + single-binary terminal server (`terminal::local_tty::{spawner,server}`), surfaced as terminal panes in `crate::pane_group`; programmatic control via `warp_cli` `local_control` + `oz agent run` (`LaunchMode::CommandLine`). |
| **De-auth (stub login seam)** | Single `auth::init(ctx)` call (`app/src/auth/mod.rs:56`) backed by `warp_server_auth` (`AuthStateProvider`, `UserUid`, `anonymous_id`); existing `loginless_conversion` feature is the scaffolding. Note release channels reject server-URL overrides. |
| **Rebrand (keep credit)** | Per-binary `src/bin/*.rs` wrappers carry the branding (`AppId`, `Info.plist`, URL schemes, "Denver Technologies" copyright); OSS channel (`warp-oss`, `default-run`) is the build a Marley fork produces. Workspace license is AGPL-3.0 + MIT `warpui*` — credit obligations live in `LICENSE-AGPL`/`LICENSE-MIT`, `about.toml`, `deny.toml`. |
