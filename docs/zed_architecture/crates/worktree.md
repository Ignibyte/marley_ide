# worktree

> Per-crate reference (Marley round 2) — crate dir `crates/worktree` (package `worktree`, lib root
> `src/worktree.rs`). Zed is the EDITOR reference for Marley's editing surface. Source cloned into session
> scratch only; this is Marley's own description, never Zed code.

| | |
|---|---|
| **Subsystem** | [06 — Project, FS, Worktree & Search](../subsystems/06-project-fs-search.md) |
| **License** | **GPL-3.0-or-later** (per-crate `LICENSE-GPL`) |
| **Provenance** | `[Zed-derived]` immutable-snapshot model, composed over `[permissive/public]` `ignore` (gitignore) |
| **Key external deps** | `ignore` (Unlicense/MIT — gitignore parsing), `chardetng`/`encoding_rs` (charset detect), `smallvec`, `parking_lot`, `postage`, `async-lock` |
| **Zed-internal deps** | `fs`, `sum_tree`, `fuzzy`, `git`, `gpui`, `settings`, `rpc`/`proto`, `text`, `language`, `clock`, `paths`, `util` |
| **Zed dependents** | ~11 crates (chiefly `project`; also `worktree_benchmarks`, panels) |
| **Marley target** | **`marley_worktree`** (new) — `Entry` + `Snapshot` + `BackgroundScanner` |

## Purpose

A `Worktree` is an **in-memory, immutable-snapshot mirror of one opened root folder**, kept current by a
background scanner reacting to fs events. It is the layer directly above `fs`: it turns "a directory on disk"
into "a cheap-to-clone, cheap-to-diff, ordered file model with stable ids and gitignore awareness" that the
finder, the tree UI, git status, and project search all read from. **Marley's `Project{root, name, is_git}`
maps to *this* crate, not to Zed's `project` hub** — see [`project.md`](./project.md).

Two ideas carry it: (1) **cheap immutable snapshots** — a copy-on-write B+-tree makes a full consistent view
publishable on *every* change and diffable in O(changed); the foreground never mutates, it only receives. (2)
**scanner decoupling** — all mutation happens on a background task; the model the UI reads is always a finished,
internally-consistent `Snapshot`.

## Key types, modules & public API

**`src/worktree.rs`** (7.1k lines).

- **`enum Worktree{ Local(LocalWorktree), Remote(RemoteWorktree) }`** (`:95`) — a gpui `Entity`; both variants
  `Deref` to a shared `Snapshot`. Public surface (`impl Worktree`, `:444`): `snapshot()`, `scan_id()` /
  `completed_scan_id()`, `abs_path()`, `is_local`/`is_remote`, `load_file`/`load_binary_file`/`write_file`,
  `create_entry`/`delete_entry`/`restore_entry`/`copy_external_entries`, `expand_entry`/`expand_all_for_entry`,
  `observe_updates`/`stop_observing_updates` (proto mirroring hooks), `wait_for_snapshot`, `is_single_file`.
  Emits **`enum Event`** (`:431`) including `UpdatedEntries` — the foreground signal the tree/finder re-render on.
- **`struct Snapshot`** (`:176`) — the shared immutable core:
  - **`entries_by_path: SumTree<Entry>`** and **`entries_by_id: SumTree<PathEntry>`** — *two indexes over the
    same files* (path-ordered traversal vs stable-id lookup).
  - `id: WorktreeId`, `abs_path`, `root_name`, `root_char_bag`, and **`scan_id` / `completed_scan_id`** counters
    (the current scan may be >1 ahead of the last fully-completed one when ops race fs events).
  - `SumTree` (Zed's own `sum_tree` crate) is a **COW B+-tree** where each node caches a composable `Summary`;
    that yields ordered/prefix traversal, O(log n) aggregate queries, stable-id lookup, **and O(1)-ish `.clone()`**
    (Arc bumps) — the linchpin that makes "publish a whole fresh snapshot on every change" cheap.
  - Read API: `entry_for_path`/`entry_for_id`, `files`/`directories`/`entries`(→ `Traversal`), `child_entries`,
    `paths`, `root_entry`/`root_dir`, `entry_count`/`visible_file_count`/… (the *aggregate* counts come straight
    off the SumTree summary), `absolutize`, `apply_remote_update` (materialize a proto diff into the tree).
- **`struct Entry`** (`:3795`) — the per-path record: `id: ProjectEntryId` (stable across renames),
  `kind: EntryKind`, `path: Arc<RelPath>`, `inode`, `mtime: Option<MTime>`, `canonical_path` (symlinks),
  `size`, `char_bag: CharBag` (lowercased chars — **fuels the fuzzy finder directly**), and status booleans
  `is_ignored` / `is_hidden` / `is_always_included` / `is_external` (symlink out of tree) / `is_private` (.env) /
  `is_fifo`. **There is deliberately NO `git_status` field** — git status is fully decoupled into a separate
  `GitStore`; the worktree tracks only *repo locations* (`LocalRepositoryEntry`, `UpdatedGitRepository`). **Copy
  this decoupling: do not put `git_status` on your entry.**
- **`enum EntryKind{ UnloadedDir, PendingDir, Dir, File }`** (`:3837`) — `UnloadedDir` is the **lazy-expansion**
  placeholder: ignored/hidden/external directories are inserted unscanned and only walked when expanded, keeping
  huge trees cheap. `enum PathChange{ Added, Removed, Updated, AddedOrUpdated, Loaded }` (`:3844`) tags each diff.
- **`struct ProjectEntryId(usize)`** (`:6713`) — monotonic stable handle; **`struct CharBag`** (from `fuzzy`) is
  the per-entry lowercased char set the finder scores against. `Traversal` (`:6442`) is the cursor that walks
  `entries_by_path` with include-ignored / start-offset options.
- **`struct LocalWorktree`** (`:132`) — owns `snapshot: LocalSnapshot`, `fs: Arc<dyn Fs>`, `settings:
  WorktreeSettings`, scan-request channels, and `_background_scanner_tasks`. **`struct LocalSnapshot`** (`:249`)
  extends `Snapshot` with gitignore state: `ignores_by_parent_abs_path`, `repo_exclude_by_work_dir_abs_path`,
  `global_gitignore`, `git_repositories: TreeMap<ProjectEntryId, LocalRepositoryEntry>`, and a
  `root_file_handle` (to re-find the root after it moves).
- **`struct BackgroundScanner`** (`:4166`) + **`enum BackgroundScannerPhase{ InitialScan,
  EventsReceivedDuringInitialScan, Events }`** (`:4187`) — owns *all* mutation on a gpui background task.
  `run()` (`:4193`) first `discover_ancestor_git_repo` (a `.git` above the root still governs ignores), does the
  **initial scan** (fans `read_dir` jobs across the executor, forcing `.git`/`.gitignore` to the front so ignore
  state is known before siblings are classified), then enters a steady **`select_biased!` loop** (explicit
  refresh requests beat fs-event noise) that coalesces bursts and calls `reload_entries_for_paths` to recompute
  only affected entries. `BackgroundScannerState` (`:270`) holds `prev_snapshot` + `RemovedEntries{by_inode,
  by_path}` (`:292`) — the inode+path bookkeeping that lets a rename **re-use** an `Entry`'s id. **Publishing** =
  clone snapshot, diff vs `prev_snapshot`, send `ScanState::Updated{snapshot, changes}` over a channel; the
  foreground installs it and emits `UpdatedEntries`. `.git`-internal churn (`*.lock`, `logs/`, `objects/`) is
  filtered to avoid needless rescans.
- **`struct RemoteWorktree`** (`:160`) — **never touches a filesystem.** It materializes `proto::UpdateWorktree`
  diffs into the *same* `Snapshot` type via a double-buffered `Arc<Mutex<(Snapshot, Vec<Update>)>>` (wire-ingest
  decoupled from model-apply). One snapshot type, two producers (fs scanner vs proto stream) — this is exactly
  what the remote subsystem mirrors.

**`src/ignore.rs`** (129 lines) — wraps the **`ignore` crate's `Gitignore`**. `struct IgnoreStack` +
`enum IgnoreStackEntry{ None, Global, RepoExclude, Some{abs_base_path, ignore, parent}, All }` is a **persistent
linked list** built top-down as the scanner descends (each dir cheaply shares its parent's stack via `Arc`).
`is_abs_path_ignored(abs_path, is_dir)` walks deepest-first, first-match-wins, honoring negation (`!pat`), git
precedence (dir-local `.gitignore` → `.git/info/exclude` → global), and short-circuits any `.git` directory.

**`src/worktree_settings.rs`** (109 lines) — `struct WorktreeSettings` layers `PathMatcher` globs *on top of*
gitignore: `file_scan_exclusions` (never scan — beats everything), `file_scan_inclusions` (scan even if
gitignored) + `parent_dir_scan_inclusions`, `private_files`, `hidden_files`, `read_only_files`, and
`scan_symlinks`. Predicates `is_path_excluded` / `is_path_always_included` / `is_path_private` / `is_path_hidden`
each walk `path.ancestors()`.

## Depends on (internal)

- [`fs`](./fs.md) — `Arc<dyn Fs>`; the scanner's entire IO (initial walk + `watch()` stream + `metadata`).
- `sum_tree` — the COW summarizing B+-tree behind `entries_by_path`/`entries_by_id` (Zed-GPL; concept is public CS).
- `fuzzy` — `CharBag` for the finder.
- `git` — `LocalRepositoryEntry` / repo discovery (locations only, *not* status).
- `settings` — `WorktreeSettings` registration; `rpc`/`proto` — remote diff wire types; `text`/`language` —
  `File`/loaded-file encoding; `gpui` — `Entity`/`Task`/`BackgroundExecutor`.

## Used by (internal)

~11 crates, dominated by [`project`](./project.md) (`WorktreeStore` holds `Vec<WorktreeHandle>`). The tree UI,
finder, and search all read `Snapshot` transitively through the project hub.

## Marley today · gap

`marley_project::FileTree` is a **static, one-shot** structure: `Project::discover_in` finds the nearest `.git`
root, `list_files_in` does a single capped (10k) `std::fs` stack walk pruning a hardcoded `should_skip` set
(`.git`/`node_modules`/`target`/`.DS_Store`), `FileTree::from_files` nests it, and `RootView` builds it **once at
boot**. Gaps vs Zed: **(a)** no fs watching — create/delete/rename on disk is invisible until relaunch; **(b)**
no real `.gitignore` (only the 4-name `should_skip`); **(c)** no stable entry ids, no incremental diff, no lazy
expansion, no per-entry metadata; **(d)** the tree and the `⌘P` finder each re-walk disk independently.

## Reimplementation on our stack — `marley_worktree`  `[Marley-original]`

Turn the static tree into a **live snapshot** over `marley_fs`. Marley does *not* need Zed's full weight (no
collab, no per-entry git status, initially no SumTree), but should adopt the core moves:

- **A snapshot over an ordered map.** Start with `Arc<BTreeMap<RelPath, Entry>>` (clone-to-publish) rather than a
  bespoke SumTree — the summary/aggregate queries (file counts) are a *later* optimization; the essential
  property is *cheap immutable publish + diff*. Keep `Entry{ id, kind, path, mtime, is_ignored, char_bag }` and
  stable `EntryId`s so the finder, tree UI, and future search share **one identity** for a file.
- **A scanner over `marley_fs`.** Initial parallel walk + a `watch()`-driven steady loop that recomputes only
  affected paths and publishes snapshots on a channel. The gpui `RootView` swaps the `Arc` and re-renders,
  reusing today's `visible_rows`/`toggle` now fed by a live snapshot instead of a boot-time `Vec`.
- **Real ignore** via the **`ignore` crate** + an `IgnoreStack`, replacing `should_skip`. Add a
  `file_scan_exclusions`-style glob setting (Marley already has `marley_settings`).
- **Fold `list_files_in` + the `⌘P` finder into the worktree** — the finder ranks `snapshot.files()` by
  `char_bag`/`nucleo` instead of re-walking. One source of truth for "the project's files."
- **Keep git-status off `Entry`** (adopt the decoupling) and **keep `UnloadedDir` lazy expansion** so a giant
  repo doesn't scan `node_modules` until expanded.

### Sequencing
1. `Entry` + `Snapshot` (immutable, clone-to-publish) + port `from_files`/`visible_rows`. Pure, cov/MSI 100,
   `FakeFs`-driven.
2. `BackgroundScanner` (initial scan only) over `marley_fs` — replace the boot-time walk; finder reads `snapshot.files()`.
3. Wire `fs.watch()` → incremental reload → live tree (create/delete/rename reflected without relaunch).
4. Swap `should_skip` → `ignore` crate + `IgnoreStack` + a `file_scan_exclusions` setting.

## Provenance

- `[permissive/public]` — **`ignore`** (gitignore parsing, Unlicense/MIT), **`globset`** (behind `PathMatcher`),
  `chardetng`/`encoding_rs`. Directly usable.
- `[Zed-derived]` (patterns, reimplement clean-room) — the immutable-snapshot-published-on-every-change model;
  the `Entry` field taxonomy incl. **git-status-decoupled**; the two-index (`by_path` + `by_id`) design; the
  scanner phases + `.git`-front-loading + lazy `UnloadedDir` expansion; the inode+path rename-id-reuse
  bookkeeping; the hierarchical `IgnoreStack` composition; the settings-globs-over-gitignore layering.
- `[permissive/public: CS literature]` — `SumTree` is a summarizing COW B-tree; the *concept* is public, Zed's
  *implementation* is GPL — reimplement (or start on a plain ordered map), don't lift.
- `[Marley-original]` — the down-scoped `marley_worktree`, and starting on `Arc<BTreeMap>` before any SumTree.

## Notes / gotchas

- **`scan_id` vs `completed_scan_id`** — a snapshot can be mid-scan; callers that need consistency await
  `completed_scan_id` (Zed's `wait_for_snapshot`/`scan_complete`). Reproduce this if you publish mid-scan.
- **Stable ids depend on inode+path bookkeeping** — a naive "clear and rebuild" scanner loses id stability across
  renames (breaks the finder's/editor's file identity). Keep `RemovedEntries{by_inode, by_path}`.
- **`.gitignore`/`.git` must be scanned first** — classify ignore state before siblings, or entries flicker
  ignored↔not. Front-load them like Zed's initial-scan job ordering.
- **Snapshot clone must stay cheap** — the whole model rests on O(1)-ish publish. On a plain `BTreeMap`, wrap in
  `Arc` and clone-the-Arc; only mutate a fresh map on the scanner side.
