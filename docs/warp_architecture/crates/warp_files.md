# warp_files

> Per-crate reference (Marley round 2). Dir: `crates/warp_files`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL]` — gap: Marley's file surface is nascent (editor buffers + the layout codecs); no ported central open/save file model. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`) |
| Internal deps | 6 |
| Used by | 1 |

## Purpose

A **central model for opening, reading, watching, and saving files** — both local and on remote hosts. It gives the app one place to load file content asynchronously, broadcast load/save/update events to subscribers, watch files for on-disk changes (individually or via repository subscriptions), and persist edits with version tracking. It also houses a text-reader that segments files by line range under a byte budget for the agent/editor surfaces.

## Key types, modules & public API

Two modules: `crates/warp_files/src/lib.rs` and `crates/warp_files/src/text_file_reader.rs`.

**`lib.rs`**
- `pub struct FileModel` — the entity (`SingletonEntity`, built via `FileModel::new(ctx: &mut ModelContext<Self>)`). Core operations:
  - Registration: `register_file_path(...)`, `register_remote_file(host_id: HostId, path: StandardizedPath) -> FileId`, `file_path(file_id) -> Option<PathBuf>`.
  - Lifecycle: `open(...)`, `save(...)`, `rename_and_save(...)`, `delete(...)`, `cancel(file_id)`, `unsubscribe(file_id, ctx)`.
  - Versioning: `set_version(file_id, ContentVersion)`, `version(file_id) -> Option<ContentVersion>`, `get_future_handle(file_id) -> Option<SpawnedFutureHandle>`.
  - Async static readers: `read_content_for_file`, `read_lines_async`, `read_text_file`, `read_file_as_binary`, `file_exists`, `create_file`, `ensure_parent_directories`.
- `pub enum FileModelEvent` — `FileLoaded`, `FailedToLoad`, `FileSaved`, `FailedToSave`, `FileUpdated`, each carrying a `FileId` (+ content/version/error). `file_id()` accessor. This is what subscribers react to.
- Internal: `enum FileBackend { Local(LocalFile), Remote { host_id: HostId, path: StandardizedPath } }` — remote files resolve a `HostRequestHandle` from `RemoteServerManager` per call (no per-file `Arc`, so disconnects fail naturally); `enum WatcherType { None, Individual, Repository }`.

**`text_file_reader.rs`**
- `pub struct TextFileSegment` — `{ file_name, content, line_range: Option<Range<usize>>, last_modified, line_count }`.
- `pub enum TextFileReadResult` — `Segments { segments, bytes_read }` or `NotText` (invalid UTF-8 → caller falls back to the binary path).
- `pub(crate) struct TextFileAccumulator` — builds segments for line ranges under `max_bytes`, normalizing `\n`/`\r\n` to LF.

## Depends on (internal)

- [`./remote_server.md`](./remote_server.md) — `RemoteServerManager` resolves per-host request handles so `FileBackend::Remote` can read/write files on remote hosts.
- [`./repo_metadata.md`](./repo_metadata.md) — `DetectedRepositories`, `Repository`, `RepositorySubscriber`, `RepositoryUpdate`, `CanonicalizedPath`: drives repository-level file watching (the `WatcherType::Repository` path).
- [`./warp_core.md`](./warp_core.md) — `HostId` to identify remote hosts.
- [`./warp_util.md`](./warp_util.md) — `ContentVersion`, `FileId`/`FileLoadError`/`FileSaveError`, and `StandardizedPath` (the platform-aware path type for remote files).
- [`./warpui_core.md`](./warpui_core.md) — the entity/model framework: `Entity`, `ModelContext`, `ModelHandle`, `SingletonEntity`, `r#async::SpawnedFutureHandle`.
- [`./watcher.md`](./watcher.md) — `BulkFilesystemWatcher` / `BulkFilesystemWatcherEvent` for individual-file watching.

(Also pulls external `notify-debouncer-full`, `async-fs`, `async-channel`, `futures`.)

## Used by (internal dependents)

- [`./warp.md`](./warp.md) — the top-level app crate is the only direct dependent; it owns the `FileModel` singleton and subscribes editor/agent surfaces to `FileModelEvent`.

## Related crates

- [`./watcher.md`](./watcher.md) and [`./repo_metadata.md`](./repo_metadata.md) — the two change-detection backends behind `WatcherType`.
- [`./warp_editor.md`](./warp_editor.md) — a primary consumer concept: editor buffers are backed by file content/versions this model produces.
- [`./remote_server.md`](./remote_server.md) — the remote-host transport for `FileBackend::Remote`.

## Marley relevance

**Classify: KEEP / EXTEND.** This crate is squarely on **goal 2 (session spawn / write / read)** — it is the canonical read/write/watch path for file content. Marley's custom panel (**goal 1**) and any session that needs to surface or persist files should route through `FileModel` rather than re-implementing async file IO, and its `FileModelEvent` stream is the natural hook for a new panel to observe loads/saves. EXTEND if Marley needs new operations (e.g. a panel-driven "open these files into the agent" flow) — add methods/events here rather than scattering `async-fs` calls. No auth or branding concerns: keep. Rename is deferred (only `warp` depends on it, but it transitively touches the heavy `warp_core`/`remote_server` stack, so churn it last).

## Notes / gotchas

- **Remote files hold no `Arc` to the host.** By design `FileBackend::Remote` stores only `host_id` + `path` and re-resolves a `HostRequestHandle` from `RemoteServerManager` at call time, so a disconnected host surfaces as a failed request rather than a leaked connection — don't "optimize" this into a cached handle.
- **Line-ending semantics:** `TextFileAccumulator` normalizes `\n` and `\r\n` to LF but does **not** treat lone `\r` (classic Mac) as a line separator, matching `read_line()`. The file's trailing-newline state is preserved via a `has_trailing_newline` flag.
- **Two watcher paths** (`Individual` vs `Repository`) must not double-watch — `WatcherType` tracks which one is active per file.
- Has a `test-util` feature and substantial test modules (`lib_tests.rs`, `text_file_reader_tests.rs`, `test_data/test_file.rs`).
