# warp_util

> Per-crate reference (Marley round 2). Dir: `crates/warp_util`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Marley-original]` — rebuilt as `marley_util` (`StandardizedPath`/`HostId`/`FileId`/`ContentVersion`/`LocalOrRemotePath`). See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`) |
| Internal deps | 2 |
| Used by | 16 |

## Purpose

The workspace's **shared grab-bag of generic utilities** — "if it's useful outside a single crate but too small to warrant its own crate, it goes here" (per the crate's own module docs). It owns the foundational value types (file ids, content versions, host ids, standardized/remote paths), path-massaging and git helpers, async/sync primitives, file-type detection, and the asset-directory constants that `asset_macro` and `warp_assets` build on. With 16 internal dependents it is one of the most widely used leaf crates in the tree.

## Key types, modules & public API

Root: `crates/warp_util/src/lib.rs` re-exports these modules:

- `assets` — directory-name constants (`ASSETS_DIR`, `BUNDLED_ASSETS_DIR`, `ASYNC_ASSETS_DIR`, `REMOTE_ASSETS_DIR`, `WINDOWS_ASSETS_DIR`, plus Windows DLL/EXE names) and URL helpers `hashed_asset_path`, `hashed_asset_url`, `make_absolute_url`. **This module is the contract `asset_macro`/`warp_assets` depend on.**
- `content_version` — `pub struct ContentVersion(usize)` with `new`, `from_raw`, `as_i32`/`as_u64`. The monotonic version stamp used by `warp_files`/editor.
- `file` — `pub struct FileId(usize)` and the error enums `FileSaveError`, `FileLoadError`.
- `file_type` — binary/text detection: `is_buffer_binary`, `is_file_content_binary`, `is_binary_file`, `is_markdown_file`, `is_jupyter_notebook_file` (uses `content_inspector` + `mime_guess`).
- `host_id` — `pub struct HostId(String)` (`new`, `as_str`); identifies a remote host. (Note: `warp_core` re-exports a `HostId` too; consumers like `warp_files` use the `warp_core` one.)
- `standardized_path` — `pub struct StandardizedPath(TypedPathBuf)` over `typed-path`, plus `InvalidPathError`; cross-platform path that can represent remote (non-host) paths: `try_new`, `try_from_local`, `from_local_canonicalized`, `as_str`, `parent`, `file_name`, `extension`.
- `local_or_remote_path` — `pub enum LocalOrRemotePath` unifying local and remote paths (`is_local`, `display_name`, `parent`, `join`, `to_local_path`, `as_remote`).
- `remote_path` — `pub struct RemotePath { host_id, path }`, `RemoteNavigationResult`.
- `path` — user-facing path utilities: `user_friendly_path` (`$HOME`-collapsing), `warp_shell_path`, `LineAndColumnArg`, `EscapeChar`, `ShellFamily`, `CleanPathResult`, and `TEST_SESSION_HOME_DIR` (a `lazy_static` test override).
- `git` — async git invocation: `run_git_command`, `run_git_command_with_env` (the latter lets hooks see user-installed binaries via a `PATH` override).
- `sync` — `pub struct Condition` (a set/reset/wait gate over `event-listener`).
- `on_cancel` — `pub trait OnCancelFutureExt` / `OnCancelFuture` (run a closure if a future is dropped before completion).
- `user_input` — `pub struct UserInput<T>` newtype marking untrusted input.
- `worktree_names` — `generate_unique_name` / `generate_worktree_branch_name`, drawing from a desert/southwest-themed word list.
- `windows` (cfg `windows` only) — Windows-specific helpers.

## Depends on (internal)

- [`./command.md`](./command.md) — used by the `git` module (`command::r#async::Command`, `command::Stdio`) to spawn git subprocesses with kill-on-drop and env control.
- [`./warpui_core.md`](./warpui_core.md) — present as a **dev-dependency** (for tests exercising the async/path helpers against the model framework); the depgraph counts it as an edge.

(External: `typed-path`, `content_inspector`, `mime_guess`, `dirs`, `regex`, `rand`, `event-listener`, `pin-project`, `dunce`, plus `windows` on Windows and `gloo` on WASM.)

## Used by (internal dependents)

16 dependents — effectively the whole core stack. Notable ones: [`./warp.md`](./warp.md), [`./warp_core.md`](./warp_core.md), [`./warp_terminal.md`](./warp_terminal.md), [`./warp_editor.md`](./warp_editor.md), [`./warp_files.md`](./warp_files.md), [`./warpui_core.md`](./warpui_core.md), [`./ai.md`](./ai.md), [`./lsp.md`](./lsp.md), [`./languages.md`](./languages.md), [`./remote_server.md`](./remote_server.md), [`./repo_metadata.md`](./repo_metadata.md), [`./syntax_tree.md`](./syntax_tree.md), [`./warp_cli.md`](./warp_cli.md), [`./warp_completer.md`](./warp_completer.md), [`./cloud_object_models.md`](./cloud_object_models.md), and [`./asset_macro.md`](./asset_macro.md).

## Related crates

- [`./asset_macro.md`](./asset_macro.md) / [`./warp_assets.md`](./warp_assets.md) — both anchor on `warp_util::assets` for directory names and hashed-asset URLs.
- [`./warp_core.md`](./warp_core.md) — the next layer up; also exposes a `HostId` and builds on these primitives.
- [`./command.md`](./command.md) — the subprocess layer `git` rides on.

## Marley relevance

**Classify: KEEP (RENAME deferred — highest-cost rename in the batch).** This is foundational, brand-neutral plumbing; almost every Marley goal touches it transitively. Specific contact points: `worktree_names` and `git` support **goal 2 (session spawn/read/write)**; `path`/`standardized_path`/`local_or_remote_path` are the value types a custom panel (**goal 1**) will pass around; `assets` is where the **goal-4 rebrand** of remote-asset URLs would land (`make_absolute_url` constructs the Warp asset CDN URL — repoint or stub it here). **Do not rename casually**: 16 dependents means `warp_util → marley_util` is the single most expensive rename in this batch; if we standardize on a `marley_*` prefix, do it as a late mechanical pass. No auth logic lives here, so **goal 3** is untouched.

## Notes / gotchas

- **Heavy cfg surface.** `windows` module + `windows` crate on Windows; `gloo` on WASM; `git` functions are `#[cfg(not(target_family = "wasm"))]`. Don't assume a symbol exists on every target.
- **Two `HostId`s exist** (here and in `warp_core`); they are distinct types — `warp_files` imports `warp_core::HostId`, not this one. Watch for confusion when refactoring.
- `git::run_git_command_with_env` sets `GIT_OPTIONAL_LOCKS=0`, `diff.autoRefreshIndex=false`, and `kill_on_drop(true)`; the `PATH` override exists specifically so LFS `pre-push` hooks can find `git-lfs` (references `specs/APP-4188`).
- `path::TEST_SESSION_HOME_DIR` is a `lazy_static` env override used to make `$HOME`-collapsing deterministic in tests.
- `worktree_names` randomness uses `rand::Rng`; `generate_unique_name` takes an explicit `rng` for reproducible tests.
