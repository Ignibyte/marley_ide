# repo_metadata

> Per-crate reference (Marley round 2) for `crates/repo_metadata`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — telemetry]` — 13k-LOC repo-tree/standing-query engine that emits Warp telemetry; Marley's `marley_project` (git-root discovery) is only the seed. Large gap. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [06 — Platform, Settings, Persistence & Infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `license`; no per-crate LICENSE marker) |
| **Internal deps** | 6 (incl. dev-deps) |
| **Used by** | 5 |

## Purpose

`repo_metadata` is the in-app model of the user's **git repositories and their file
trees**. It detects whether a path lives inside a repo, builds and incrementally
maintains a file tree (honouring `.gitignore`), watches the filesystem for changes, and
exposes the resulting state as a reactive `warpui_core` model that the rest of the app
queries. It is the data source behind file-path autocomplete, the file tree / project
panels, AI context gathering, and "standing queries" (saved searches kept live as files
change). It supports both **local** repositories and **remote** ones (snapshots streamed
from a `remote_server` over SSH).

## Key types, modules & public API

- **`lib.rs`** — `RepoMetadataError` (the crate error enum), `is_in_repo(path, app) -> bool`,
  and `CanonicalizedPath` (a `PathBuf` newtype canonicalized via `dunce::canonicalize`,
  with `Borrow<Path>`/`Borrow<PathBuf>` for use as a map key).
- **`wrapper_model.rs`** — `RepoMetadataModel` is the top-level entry point a consumer
  holds. `RepoMetadataModel::new(ctx)` / `new_with_incremental_updates(ctx)` construct it;
  methods include `get_repository`, `repository_state`, `repository_indexed`,
  `index_local_directory_path`, `index_directory`, `load_directory`,
  `find_repository_for_path`, `insert_remote_snapshot`, `apply_remote_incremental_update`,
  `register_force_included_paths`, `set_project_skill_provider_paths`. Emits
  `RepoMetadataEvent`. It wraps the local + remote sub-models.
- **`local_model.rs`** — `LocalRepoMetadataModel`, `RepoContent<'a>` / `RepoContents<'a>`
  (borrowed views over a repo's entries), and `RepositoryMetadataEvent`.
- **`remote_model.rs`** — `RemoteRepoMetadataModel` for SSH-streamed remote repos.
- **`repository.rs`** — `Repository` (`root_dir`, `git_dir`, `common_git_dir`,
  `start_watching`/`stop_watching`, `check_gitignore_status`), the `RepositorySubscriber`
  trait + `BufferingRepositorySubscriber<S>` debounce wrapper, and `StartWatching`.
- **`repositories.rs`** — `DetectedRepositories` (`detect_possible_git_repo`,
  `get_root_for_path`, `register_remote_repo_root`, `remove_roots_for_host`) and
  `RepoDetectionSource` / `DetectedRepositoriesEvent`.
- **`entry.rs`** — `Entry`, `DirectoryEntry`, `FileId`, `FileMetadata`, `BuildTreeError`,
  and the gitignore helpers `gitignores_for_directory`, `matches_gitignores`,
  `should_ignore_git_path`.
- **`file_tree_store.rs` / `file_tree_update.rs`** — `FileTreeEntry`, `RepoMetadataUpdate`,
  `MetadataUpdateType` (the incremental tree-diff plumbing).
- **`standing_queries.rs`** — `StandingQueryDefinitions`, `StandingQueryResults`,
  `StandingQueryResultsDelta` (live saved-search results).
- **`watcher.rs`** — `DirectoryWatcher`, `RepositoryUpdate`, `TargetFile`.

Feature flags: `local_fs`, `test-util`. WASM builds stub out the watcher/filesystem
(`is_in_repo` returns `false`; `notify-debouncer-full`/`async-fs`/`watcher` are
`cfg(not(wasm))` only).

## Depends on (internal)

- [./warp_core.md](./warp_core.md) — core app types and `warp_core::telemetry` events that repo indexing emits.
- [./warp_util.md](./warp_util.md) — `StandardizedPath`, `LocalOrRemotePath`, path standardization used throughout.
- [./warpui_core.md](./warpui_core.md) — the reactive model framework (`AppContext`, `ModelContext`, `SingletonEntity`) this crate's models are built on.
- [./watcher.md](./watcher.md) — low-level filesystem watch primitive (native only).
- [./warpui.md](./warpui.md) — UI/test-util integration (dev-dependency, `test-util` feature).
- [./virtual-fs.md](./virtual-fs.md) — in-memory filesystem used in tests (dev-dependency).

> Note: `warpui` and `virtual-fs` appear in the depgraph edge set but are **dev-dependencies**
> in `Cargo.toml`; the production deps are `warp_core`, `warp_util`, `warpui_core`, `watcher`.

## Used by (internal dependents)

- [./warp.md](./warp.md) — the main binary wires `RepoMetadataModel` into the app.
- [./ai.md](./ai.md) — gathers repo file context for prompts.
- [./lsp.md](./lsp.md) — maps repo file trees to language-server roots.
- [./remote_server.md](./remote_server.md) — produces the remote snapshots consumed by `RemoteRepoMetadataModel`.
- [./warp_files.md](./warp_files.md) — file tree / picker UI.

Total: **5** dependents.

## Related crates

- [./watcher.md](./watcher.md) and [./virtual-fs.md](./virtual-fs.md) — its filesystem layer.
- [./warp_search_core.md](./warp_search_core.md) and [./warp_ripgrep.md](./warp_ripgrep.md) — content search, complementary to repo_metadata's path/tree indexing; standing queries bridge the two.
- [./warp_files.md](./warp_files.md) — the primary UI consumer.

## Marley relevance

**KEEP.** This is core, non-auth, non-brand infrastructure: it is how Marley knows what
files and repos exist, which is the substrate for goal (1) a custom panel (a project /
file panel needs this tree) and goal (2) session spawn/read (a spawned session inherits a
repo root and its tree). It does not gate on login, so de-auth (goal 3) is not blocked
here. The only de-Warp (goal 4) touchpoints are cosmetic: the package name `repo_metadata`
is already neutral (no rename needed), but it emits `warp_core::telemetry` events on
indexing — when we stub telemetry centrally in `warp_core`, those calls become no-ops, so
nothing changes here. Remote-repo support (`RemoteRepoMetadataModel`,
`remove_roots_for_host`) is tied to Warp's SSH/`remote_server` feature; if Marley ships
local-only first, leave it compiled but unused rather than ripping it out (5 dependents
make surgery risky).

## Notes / gotchas

- Heavy `cfg(target_family = "wasm")` split: the watcher, `async-fs`, and `notify-debouncer-full` are native-only; `is_in_repo` and much of the tree machinery degrade to stubs on WASM. Test both targets after edits.
- `CanonicalizedPath` implements `Borrow<Path>`/`Borrow<PathBuf>` and relies on those types having *identical* `Hash`/`Ord`/`Eq` — the safety comment in `lib.rs` warns that diverging impls would make the `Borrow` unsound (don't add custom comparison logic).
- The depgraph edge set mixes in dev-dependencies (`warpui`, `virtual-fs`); the real runtime fan-in is smaller.
- `build.rs` present (uses `anyhow` build-dep) — there is generated build-time work; check it before assuming a pure-source crate.
