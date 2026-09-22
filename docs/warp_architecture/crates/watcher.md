# watcher

> Per-crate reference (Marley round 2) — crate dir `crates/watcher`. Marley is Ignibyte's fork of Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — debounced FS watcher over `notify_debouncer_full`; standard concept. Gap. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (no in-crate LICENSE marker; inherits workspace `license`) |
| Internal deps | 1 (`warpui_core`) |
| Used by | 4 (`ai`, `repo_metadata`, `warp`, `warp_files`) |

## Purpose

A debounced **filesystem watcher** exposed as a `warpui_core` model/entity. It runs `notify` + `notify-debouncer-full` on a dedicated background thread, normalizes the platform-specific raw event stream into a clean add/modify/delete/move diff, and emits that to subscribers on the UI/model loop. It exists so higher-level features (file tree, repo metadata, AI context) can react to disk changes without each one re-implementing cross-platform watching and event de-noising.

## Key types, modules & public API

Two modules: `src/lib.rs` (the bulk watcher) and `src/home_watcher.rs`.

- `pub struct BulkFilesystemWatcher` — a `warpui_core::Entity` (`type Event = BulkFilesystemWatcherEvent`). Constructed with `new(debounce_duration: Duration, ctx: &mut ModelContext<Self>)`; spawns the named `"Bulk Filesystem Watcher"` background thread and bridges it to the model loop via `async_channel`.
  - `register_path(&mut self, path, watch_filter: WatchFilter, recursive_mode: RecursiveMode) -> impl Future<Output = Result<()>>`
  - `unregister_path(&mut self, path) -> impl Future<Output = Result<()>>` (awaiting the future is optional; registration happens regardless)
  - `new_for_test()` — stub with a dead command channel, no thread.
- `pub struct BulkFilesystemWatcherEvent` — `added`, `modified`, `deleted: HashSet<PathBuf>` and `moved: HashMap<PathBuf, PathBuf>` (target → source). Helpers `added_or_updated_iter()` / `added_or_updated_set()`.
- `pub use home_watcher::{HomeDirectoryWatcher, HomeDirectoryWatcherEvent}` — a `SingletonEntity` that watches `$HOME` non-recursively with a 500ms debounce and re-emits `HomeDirectoryWatcherEvent::HomeFilesChanged(BulkFilesystemWatcherEvent)`.
- Internal: `BackgroundFileWatcher` (owns the `Debouncer<RecommendedWatcher, NoCache>`, command loop over `mpsc`), `WatcherEventHandler` (impls `DebounceEventHandler`), and `deduplicate_and_merge_raw_notifier_events` — the rename/create/remove squashing logic (handles macOS trash-as-rename and Windows `ModifyKind::Any`).

## Depends on (internal)

- [`warpui_core`](./warpui_core.md) — the UI/model runtime: `Entity`, `ModelContext`, `ModelHandle`, `SingletonEntity`. The watcher *is* a model and emits events through `ctx.emit` / `ctx.spawn_stream_local`.

## Used by (internal dependents)

- [`warp`](./warp.md) — top-level app wiring.
- [`warp_files`](./warp_files.md) — file-tree / file panel reacting to disk changes.
- [`repo_metadata`](./repo_metadata.md) — invalidates git/repo metadata on change.
- [`ai`](./ai.md) — keeps AI context fresh as watched files change.

## Related crates

- [`warpui_core`](./warpui_core.md) — the entity/model framework it plugs into.
- [`warp_files`](./warp_files.md) / [`repo_metadata`](./repo_metadata.md) — primary consumers of its event diff.
- [`asset_cache`](./asset_cache.md) — sibling `warpui_core`-based infra crate in this subsystem.

## Marley relevance

**Classification: KEEP.** Brand-neutral, auth-free, genuinely reusable. It is directly relevant to Marley goal (1) expand the UI surface with a custom panel: a Marley panel that shows files, repo state, or live context can subscribe to `BulkFilesystemWatcher`/`HomeDirectoryWatcher` exactly like `warp_files` does. No rename needed (package is already generic `watcher`). The only Marley touchpoint is its dependency on `warpui_core` (MIT) — which we keep as the rendering/model substrate. Leave the dedup/rename-squashing logic alone; it encodes hard-won cross-platform quirks. KEEP as-is.

## Notes / gotchas

- **Dedicated OS thread + channel bridge:** the `notify` watcher lives on a background thread because register/unregister do blocking fs calls; results flow back via `async_channel` to `spawn_stream_local`. If the thread fails to spawn, watching is silently disabled (logged) and the app continues.
- **Event normalization is lossy by design:** rename ordering isn't guaranteed by `notify`, so rapid `A→B→C` renames may degrade to delete+create based on current fs state (see the inline `TODO(kevin)` and the `RenameMode::Any` / macOS-trash handling).
- Platform specifics baked in: Windows `ReadDirectoryChangesW` emits `ModifyKind::Any`; macOS move-to-trash arrives as a `Modify(Name(Any))`.
- `edition = "2021"`; authors still listed as `Warp Team <dev@warp.dev>` in `Cargo.toml` (cosmetic de-Warp target if doing goal 4 thoroughly).
