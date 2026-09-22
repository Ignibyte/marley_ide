# Subsystem 06 — Platform, Settings, Persistence & Infra

> Part of the Marley architecture docs. Marley is forked from Warp (warpdotdev/warp).
> **Round 3 re-review (2026-07-12):** verified against the shipped tree. This subsystem is now
> **the furthest along of any** — Marley has shipped its **own** settings framework (`marley_settings`),
> feature-flag registry + path/config layer (`marley_core`), value vocabulary (`marley_util`), and
> layout persistence (the grid/shell codecs in `marley_app/grid_layout.rs`). These are Marley-original
> reimplementations, **not** ports of the Warp crates described below. See the new
> [**Marley status @ M15**](#marley-status--m15) and [**Provenance & licensing**](#provenance--licensing)
> sections, and the one-line provenance tag now atop each crate doc in this subsystem.

This subsystem is the broad infrastructure grab-bag that underpins every other crate:
where configuration, feature flags, and persisted state live; how the app talks to the
filesystem, the OS, and bundled assets; logging/telemetry plumbing; channel/version
metadata; and a set of small leaf utilities (vim motions, handlebars, ipynb parsing,
voice capture, search). The two pieces that matter most for **Marley's de-auth and
rebrand goals** are `settings` (the config + sync seam) and `persistence` (the SQLite
schema that holds user/team/account state). Telemetry plumbing (`warp_logging` Sentry,
`warp_core::telemetry` events surfaced by `repo_metadata`/`warp_search_core`) is the
second rebrand/de-auth concern.

All paths below are relative to the repo root
(`/Users/chadpeppers/Projects/ignibyte/Marley`). Crate dirs live under `crates/`.

---

## A. Settings & feature flags (PRIMARY rebrand/de-auth surface)

### `settings` (`crates/settings`, ~4.3k LOC)
The central settings framework. Settings are declared via macros
(`define_settings_group!`, `define_setting!`, `implement_setting_for_enum!` in
`src/macros.rs`) and registered globally through the `inventory` crate. Key types:

- `SettingsManager` (`src/manager.rs`) — singleton (`warpui_core::SingletonEntity`) that
  holds per-`storage_key` maps of `update_fns`, `clear_fns`, `load_fns`, `equals_fns`,
  `is_syncable_fns`, and emits `SettingsEvent` (e.g. `LocalPreferencesUpdated`). It is the
  bridge between the declarative setting macros and the cloud-sync machinery, deliberately
  decoupled so macros don't depend on cloud code.
- `PublicPreferences` / `PrivatePreferences` (`src/lib.rs`) — newtype wrappers around
  `Box<dyn warpui_extras::user_preferences::UserPreferences>`. **Public** settings
  (`private: false`) go to the user-visible TOML settings file when the `SettingsFile`
  `FeatureFlag` is on, otherwise to the platform-native store; **private** settings
  (`private: true`) always go to the platform-native store (macOS `UserDefaults`, Windows
  registry, etc.).
- `SyncToCloud` { `Never`, `Globally(RespectUserSyncSetting)`, `PerPlatform(...)` } and
  `SupportedPlatforms` { `ALL`, `DESKTOP`, `MAC`, `LINUX`, `WINDOWS`, `WEB`, `OR(..)` } —
  per-setting policy controlling whether a value is pushed to Warp's cloud preference
  service and on which platforms it applies.
- Secret access goes through `warpui_extras::secure_storage` (imported in `src/lib.rs` as
  `AppContextExt`).

**Marley relevance (HIGH):** This is where config, secrets, and the cloud-sync seam all
converge. `SyncToCloud` and `secure_storage` are the hooks that talk to Warp's account
backend; for de-auth, settings marked `Globally`/`PerPlatform` are exactly the values that
would otherwise round-trip through Warp's servers. The `SettingsFile` feature flag governs
whether config is a local TOML file vs the cloud/native store.

### `settings_value` (`crates/settings_value`, ~400 LOC) & `settings_value_derive` (~500 LOC)
`settings_value` defines the `SettingsValue` trait — a parallel serialization path to serde
(`to_file_value` / `from_file_value`) producing human-friendly JSON that the TOML backend
converts to TOML. Used **only** for the user-visible settings file; cloud sync and native
stores keep using serde directly. `settings_value_derive` is the proc-macro that derives it.

### `warp_features` (`crates/warp_features`, ~1.3k LOC)
The feature-flag registry. `enum FeatureFlag` (`src/lib.rs`) enumerates ~hundreds of flags;
flags are bucketed into `LOCAL_FLAGS`, `DOGFOOD_FLAGS`, `PREVIEW_FLAGS`, `RELEASE_FLAGS`,
`RUNTIME_FEATURE_FLAGS`. API: `FeatureFlag::is_enabled()`, `set_enabled()`,
`set_user_preference()`, `override_enabled() -> OverrideGuard`. State is held in
process-global `AtomicTriState` (`Unset`/`true`/`false`); a test-only `overrides` module
provides thread-local overrides.

**Marley relevance (HIGH):** Many flags are account/login-gated — e.g.
`CreatingSharedSessions` ("enabled if the logged-in user is part of a paying team or the
allowlist"), `CloudObjects`, `AgentMode`, `CrashReporting`, `WithSandboxTelemetry`,
`RecordAppActiveEvents`, `AgentModeAnalytics`, `FetchChannelVersionsFromWarpServer`. These
are the levers for stubbing out login (force-enable locally) and for disabling telemetry.

---

## B. Persistence (PRIMARY de-auth surface)

### `persistence` (`crates/persistence`, ~2.4k LOC)
The on-disk SQLite database layer, built on **Diesel** with embedded migrations
(`MIGRATIONS = diesel_migrations::embed_migrations!("migrations")`, gated by the `local_fs`
feature). `src/schema.rs` is the Diesel table schema; `src/model.rs` holds the
`Queryable`/`Insertable` structs (named after tables). Migrations under
`crates/persistence/migrations/` date back to 2021-10.

Notable tables (from `model.rs`): `app`, `windows`, `tabs`, `tab_groups`, `panes`/`panels`,
`blocks`, `commands`, `workflows`, `notebooks`, `mcp_server_installations`,
`agent_conversations`, `agent_tasks`, `ai_*_panes` — **and account/identity tables:**
`current_user_information`, `user_profiles`, `teams`, `team_members`, `team_settings`,
`workspaces`, `workspace_teams`, `server_experiments`. Token-usage accounting fields exist
(`warp_tokens`, `byok_tokens`, `custom_endpoint_tokens`, `warp_token_usage_by_category` in
`model.rs`). Depends on `warp_multi_agent_api` for response/event types.

**Marley relevance (HIGH):** This is the local mirror of the user's account, teams, and
server experiments. De-auth work needs to decide what `current_user_information`/`teams`
look like when there is no Warp login (stub a local/anonymous user). The DB file path
itself is resolved upstream (warpui_extras/warp_core), not here.

---

## C. Filesystem, OS platform & process infra

| Crate | What it is (1-3 lines) | Marley relevance |
|---|---|---|
| `warp_files` (`crates/warp_files`, ~2.1k LOC) | Central file model for opening/saving files; subscribers watch loaded content and request saves. Includes `text_file_reader.rs`. | Supports the editor/UI surface; session file I/O. |
| `virtual_fs` (`crates/virtual_fs`, ~200 LOC) | Test-only in-memory/temp VFS (`VirtualFS` over `tempfile::TempDir`). | Test infra only. |
| `watcher` (`crates/watcher`, ~450 LOC) | Debounced filesystem watcher (`notify_debouncer_full`); `HomeDirectoryWatcher`/`HomeDirectoryWatcherEvent` in `home_watcher.rs`; runs as a `warpui_core` Entity. | Live config/settings reload; repo file change detection. |
| `node_runtime` (`crates/node_runtime`, ~550 LOC) | Installs/manages a pinned Node.js/npm (`NODE_VERSION = v22.12.0`, min v20) across macOS/Linux/Windows; gated by `local_fs`. | Needed for MCP servers / JS tooling spawned by the app. |
| `isolation_platform` (`crates/isolation_platform`, ~375 LOC, pkg `warp_isolation_platform`) | Sandbox abstraction for running Warp inside Docker (`docker.rs`, `docker_sandbox.rs`), Kubernetes (`kubernetes.rs`), or Linux namespaces (`namespace.rs`). Reads `WARP_ISOLATION_PLATFORM` / `WARP_WORKLOAD_TOKEN` env vars. | Relevant to remote/agentic session spawning in isolated environments. |
| `prevent_sleep` (`crates/prevent_sleep`, ~260 LOC) | Cross-platform "keep the system awake" guard (`mac.rs`/`windows.rs`/`noop.rs`), returns a `Guard`. | Keep long agent runs alive. |
| `app-installation-detection` (`crates/app-installation-detection`, ~56 LOC) | Tiny **axum** router exposing `GET /install_detection` (returns `"ok"`); CORS-allowlists `localhost:8080/8082`, `warp.dev` and `*.warp.dev`. | **Rebrand/de-auth:** hardcoded `warp.dev` CORS origins; this is the local endpoint the marketing site probes to detect an install. |

---

## D. Assets

| Crate | What it is | Marley relevance |
|---|---|---|
| `warp_assets` (`crates/warp_assets`, ~26 LOC) | `rust_embed` `RustEmbed` provider embedding `app/assets` (`bundled/**`, `async/**`); implements `warpui_core::AssetProvider`. | Holds branded imagery/icons — **rebrand** touch point. |
| `asset_macro` (`crates/asset_macro`, ~156 LOC) | Macros `bundled_asset!`, `remote_asset!`, `bundled_or_fetched!`; classify assets as bundled / remote-fetched / split (bundled-native, remote-web). | Rebrand asset references. |
| `asset_cache` (`crates/asset_cache`, ~330 LOC) | URL-keyed async asset cache over `warpui_core::assets::asset_cache`; fetches via `reqwest`, hashes/base64 for cache keys. | Remote asset delivery. |

---

## E. Logging, telemetry & channels (SECONDARY de-auth/rebrand surface)

| Crate | What it is | Marley relevance |
|---|---|---|
| `warp_logging` (`crates/warp_logging`, ~1.5k LOC) | Logger init (`LogConfig`, `LogDestination`). `native.rs` wires **Sentry** crash/error reporting via `sentry_log::SentryLogger` + `sentry_log_filter` (consults `warp_core::errors::should_ignore_log_for_sentry`); `rotation.rs` rotates `warp.log` and the telemetry file. WASM path in `wasm.rs`. | **De-auth/telemetry:** Sentry is the error pipeline to Warp; rebrand needs its DSN/init swapped or disabled (also gated by `CrashReporting`/`LogExpensiveFramesInSentry` flags). |
| `simple_logger` (`crates/simple_logger`, ~1k LOC) | Lightweight async, file-based stderr logger with size-based `RotationConfig`; for server processes. | Local-only logging, low rebrand risk. |
| `repo_metadata` (`crates/repo_metadata`, ~13k LOC) | Largest crate here: builds/maintains repo file trees, gitignore handling, fs watching, and standing queries (`local_model.rs`, `remote_model.rs`, `wrapper_model.rs`, `repositories.rs`, `entry.rs`). `telemetry.rs` registers `RepoMetadataTelemetryEvent` via `warp_core::register_telemetry_event`. | Powers repo-aware UI/agent context; emits telemetry events (review for de-auth). |
| `warp_search_core` (`crates/warp_search_core`, ~4k LOC) | Generic search framework over **tantivy** (native): `searcher.rs`, `mixer.rs`, `data_source.rs`, `result_renderer.rs`, plus `telemetry.rs`. | Universal-search backend; emits telemetry events. |
| `channel_versions` (`crates/channel_versions`, ~780 LOC) | `ChannelVersions` (dev/preview/stable) + changelogs; parses/compares version metadata; `overrides.rs` for local override of channel versions. | Auto-update version feed (tied to `FetchChannelVersionsFromWarpServer`). |
| `warp_channel_config` (`crates/warp_channel_config`, ~113 LOC) | `load_config!` macro loads per-channel `warp_core::channel::ChannelConfig` — embedded at compile time for `release_bundle`, else shells out to a `warp-channel-config` generator binary on PATH. | **Rebrand:** channel identity (dev/preview/stable branding) is configured here. |

---

## F. Small leaf utilities

| Crate | What it is | Marley relevance |
|---|---|---|
| `warp_util` (`crates/warp_util`, 24 files, ~4k LOC) | Cross-cutting helpers: `StandardizedPath`, `RemotePath`/`LocalOrRemotePath`, `HostId` (opaque remote-host id from server `InitializeResponse`), `git.rs`, `worktree_names.rs`, `content_version.rs`, `file_type.rs`, `on_cancel.rs`, `sync.rs`, `user_input.rs`, `windows.rs`, `assets.rs`. | Foundational; `HostId` ties into remote/session dedup. |
| `warp_js` (`crates/warp_js`, ~450 LOC) | Rust↔JavaScript value/function conversion helpers (`convert.rs`, `js_function.rs`). | JS interop for embedded scripting. |
| `handlebars` (`crates/handlebars`, ~610 LOC) | Minimal handlebars-style template argument parser (`get_arguments`, `ParsedArgumentsIterator`); not the upstream `handlebars` crate. | Workflow/command argument templating. |
| `field_mask` (`crates/field_mask`, ~170 LOC) | Applies protobuf `FieldMask` operations to `prost_reflect::DynamicMessage`. | gRPC/proto partial-update plumbing. |
| `warp_ripgrep` (`crates/warp_ripgrep`, ~360 LOC) | Thin wrapper around ripgrep for the Warp CLI (`search.rs`, `types.rs`). | Code/text search backing. |
| `ipynb_parser` (`crates/ipynb_parser`, ~810 LOC) | Render-only `.ipynb` (Jupyter v4) → `FormattedText` via `markdown_parser`; does not execute or round-trip edits. | Notebook viewing in the UI surface. |
| `vim` (`crates/vim`, 19 files, ~4.7k LOC) | Vim motion/text-object engine (matching brackets, paragraph/word iterators, find-char, registers). Pure editor logic. | Editor input mode; UI-surface feature. |
| `voice_input` (`crates/voice_input`, ~430 LOC) | Microphone capture via `cpal`, resampling via `rubato` to 16 kHz mono for the Wispr voice backend; `VoiceInput` is a `warpui_core` Entity using `MicrophoneAccessState`. | Optional input modality; sends audio to an external (Wispr) service — review for de-auth. |

---

## Cross-crate dependency notes
- `settings` depends on `warp_features` (gates), `settings_value`, `warpui_core`,
  `warpui_extras` (`secure_storage`, `user_preferences`) — the settings stack is tightly
  coupled to the UI core/extras crates (subsystem covering `warpui*`).
- `persistence` depends on `warp_multi_agent_api` (agent response/event models) and Diesel.
- `repo_metadata`, `warp_search_core`, and `warp_logging` all emit through
  `warp_core::telemetry` / Sentry — the telemetry seam lives in `warp_core`, not here.
- `warp_channel_config` / `channel_versions` depend on `warp_core::channel::ChannelConfig`.

## Where config & secrets physically live
- **User-visible settings:** TOML file (path resolved in `warpui_extras`/`warp_core`), used
  only when the `SettingsFile` feature flag is on.
- **Private settings & secrets:** platform-native store (macOS `UserDefaults` / Windows
  registry) via `warpui_extras::secure_storage`.
- **Structured state (windows/tabs/blocks/account/teams):** SQLite via `persistence`
  (Diesel migrations under `crates/persistence/migrations/`).
- **Cloud-synced settings:** those with `SyncToCloud::Globally`/`PerPlatform` round-trip
  through Warp's preference service (handled by the cloud_objects / server-client crates).

---

## Marley status @ M15

This is the **least GPL-sensitive** subsystem and the one Marley has advanced furthest on: the
settings + persistence pieces are already **rebuilt from scratch**, so the Warp crates above are
reference-only for this area. Everything Marley boots offline — no cloud, auth, telemetry, or
account DB.

### Built — Marley-original reimplementations

| Warp crate(s) | Marley reimplementation | Notes |
|---|---|---|
| `settings` + `settings_value` + `settings_value_derive` | **`marley_settings`** (`crates/marley_settings`) — the typed declarative TOML framework (M1.B) | `Setting` trait keyed by dotted `toml_path`; a `SettingsManager` over a retained `toml::Table` with lazy resolution, persist, caller-driven reload, and drop-scoped change subscriptions. **`SettingsValue` is a blanket impl over `Serialize + DeserializeOwned`** — so the whole `settings_value_derive` proc-macro is obviated. **Local TOML only; every setting non-syncable** (charter) — none of Warp's `SyncToCloud`/`SupportedPlatforms`/`secure_storage`/native-store machinery. |
| `warp_features` | **`marley_core::FeatureFlag`** (`crates/marley_core/features.rs`) | A three-layer tri-state gate (thread-local override → user-pref → global baseline → `false`), sized from the variant set at compile time. Four flags (`CommandBlocks`, `AgentMode`, `SessionRelay`, `ThemeStudio`); `apply_default_flags`/`mark_initialized` startup seam. **No account/login-gated flags.** |
| `warp_util` | **`marley_util`** (`crates/marley_util`) | The workspace value vocabulary: `StandardizedPath`/`PathFlavor`/`LocalOrRemotePath` (over `typed-path`), `HostId`, `FileId`, `ContentVersion`. Sole owner per seam-contracts. |
| `persistence` **(layout half only)** | **the grid/shell text codecs** in `crates/marley_app/grid_layout.rs` (#163/#205/#243/#258) | Pure serialize/parse of the pane grid + whole-shell layout (projects→tabs→panes, per-terminal cwd, per-CodeView file path) to a small framing-safe blob. **Deliberately NOT a SQLite/Diesel DB and NOT an account/teams store.** |
| (channel identity) | **`marley_core::Config`** (`crates/marley_core/config.rs`) | One canonical single-channel config (`net.ignibyte.marley`, `marley.log`) with **no** server/RTC/cloud/telemetry/auth fields — replaces `warp_channel_config` + `channel_versions`. |
| (path layout) | **`marley_core` paths** (`crates/marley_core/paths.rs`) | `~/.marley/{config,data,themes,skills}`, `mcp.json`, logfile path — never-panicking (`Result<_, PathError>`). |

### Adjacent / partial

- **`marley_search_core`** is a `nucleo` **fuzzy matcher** (command-palette + file-open ranking) — the
  counterpart to Warp's small `fuzzy_match` crate, **not** to `warp_search_core` (the tantivy universal-search
  framework, which has no Marley equivalent).
- **`marley_project`** (git-root discovery) is the seed of a repo/project surface but is a tiny fraction of
  Warp's 13k-LOC `repo_metadata` (repo file trees, standing queries, telemetry).

### Gaps (no Marley equivalent yet) — and which are likely N/A

- **Standard-dep wrappers, buildable on demand:** `prevent_sleep` (keep-awake), `watcher` (FS watch),
  `node_runtime` (Node installer), `simple_logger` (file logger). Standard concepts — low risk, deferred.
- **Editor/UI-surface leaves:** `warp_files` (central open/save model), `ipynb_parser`, `handlebars`
  (arg templating), `vim` (tracked under subsystem 02).
- **Telemetry / de-auth surface (intentionally absent):** `warp_logging` (Sentry), `repo_metadata`/
  `warp_search_core` telemetry events, `voice_input` (external Wispr), `channel_versions`
  (`FetchChannelVersionsFromWarpServer`) — Marley emits none of these.
- **Rebrand / N/A-offline:** `app-installation-detection` (hardcoded `warp.dev` CORS), `warp_assets` +
  `asset_macro` + `asset_cache` (branded/remote assets), `field_mask` + `warp_js` (proto/JS plumbing),
  `warp_isolation_platform` (Docker/K8s sandbox — relevant later for remote/agent isolation).

## Provenance & licensing

Warp is **AGPL-3.0**; "Marley is forked from Warp." Posture: the editor/terminal layer may carry
AGPL-derived code (kept auditable), while the sold **brain/agent layer must stay clean** of any
Warp-derived code. Every crate doc in this subsystem now carries a one-line provenance tag using the
three-tag scheme below. **Settings/persistence are the least GPL-sensitive tier — Marley has already
rebuilt them original, so no Warp copyleft attaches to Marley's config/persistence surface.**

**Tag legend**

- **`[Marley-original]`** — Marley shipped its own from-scratch implementation; the Warp crate is
  reference-only. Carries **no** Warp copyleft. → `marley_settings`, `marley_core::FeatureFlag`,
  `marley_util`, `marley_core::Config`, and the grid/shell persistence codecs.
- **`[permissive: standard deps]`** — the Warp crate is a thin wrapper over a well-known permissive
  dependency or a standard OS API; the capability is public/standard and Marley can adopt the same dep
  with negligible copyleft exposure. → `prevent_sleep`, `node_runtime`, `virtual-fs`, `warp_ripgrep`,
  `field_mask`, `asset_cache`, `watcher`, `simple_logger`.
- **`[Warp-derived/AGPL]`** — substantial Warp-original design/logic (and often a telemetry, rebrand, or
  de-auth concern); reusing the source is copyleft, so Marley must clean-room any capability it wants.
  → `warp_logging`, `repo_metadata`, `warp_search_core`, `channel_versions`, `warp_channel_config`,
  `app-installation-detection`, `warp_assets`, `asset_macro`, `voice_input`, `warp_isolation_platform`,
  `warp_files`, `ipynb_parser`, `handlebars`, `warp_js`, and Warp's `persistence` account/teams DB
  (deliberately not ported).

**Per-area summary**

| Area | Warp crate(s) | Provenance | Marley posture |
|---|---|---|---|
| Settings framework | `settings`, `settings_value(_derive)` | **`[Marley-original]`** | Rebuilt as `marley_settings` (M1.B). Done. |
| Feature flags | `warp_features` | **`[Marley-original]`** | Rebuilt as `marley_core::FeatureFlag`. Done. |
| Value vocabulary | `warp_util` | **`[Marley-original]`** | Rebuilt as `marley_util`. Done. |
| Layout persistence | `persistence` (layout half) | **`[Marley-original]`** | The grid/shell codecs. Done. |
| Account/teams DB | `persistence` (Diesel/SQLite) | `[Warp-derived/AGPL]` | **Deliberately not ported** — no login, no account/teams store. |
| Channel/config identity | `warp_channel_config`, `channel_versions` | `[Warp-derived/AGPL — rebrand/de-auth]` | Obviated by the single offline `marley_core::Config`. |
| Telemetry / crash reporting | `warp_logging`, `*::telemetry` | `[Warp-derived/AGPL — de-auth]` | Absent by design; only a local `marley.log`. |
| Standard-dep infra | `prevent_sleep`, `watcher`, `node_runtime`, `simple_logger`, `virtual-fs`, `warp_ripgrep`, `field_mask`, `asset_cache` | `[permissive: standard deps]` | Buildable on demand from the same public deps; low risk. |
| Repo/search engines | `repo_metadata`, `warp_search_core` | `[Warp-derived/AGPL]` | Gaps; `marley_project`/`marley_search_core` are only partial/adjacent seeds. |
| Assets / rebrand | `warp_assets`, `asset_macro`, `app-installation-detection` | `[Warp-derived/AGPL — rebrand]` | Rebrand touch points; not ported. |
| Editor-surface leaves | `warp_files`, `ipynb_parser`, `handlebars`, `warp_js`, `voice_input`, `warp_isolation_platform` | `[Warp-derived/AGPL]` | Gaps / partly N/A offline. |
