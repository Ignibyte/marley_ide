# fs

> Per-crate reference (Marley round 2) — crate dir `crates/fs` (package `fs`, lib root `src/fs.rs`).
> Zed is the EDITOR reference for Marley's editing surface. Source cloned into session scratch only; this is
> Marley's own architecture description, never Zed code.

| | |
|---|---|
| **Subsystem** | [06 — Project, FS, Worktree & Search](../subsystems/06-project-fs-search.md) |
| **License** | **GPL-3.0-or-later** (explicit per-crate `LICENSE-GPL`) |
| **Provenance** | `[Zed-derived]` trait+DI *shape*, composed over `[permissive/public]` `notify` (fs watching) + `trash` |
| **Key external deps** | `notify` (MIT/Apache), `ignore` (Unlicense/MIT), `trash` (MIT, Zed fork), `is_executable`, `smol`, `async-trait`, `parking_lot`, `rope`, `tempfile`; macOS `unicode-normalization`; Linux/BSD `ashpd` (portal trash); Windows `windows`/`dunce` |
| **Zed-internal deps** | `gpui`, `git`, `paths`, `util`, `proto`, `text`, `collections`, `telemetry` |
| **Zed dependents** | ~56 crates (the single most foundational IO seam) |
| **Marley target** | **`marley_fs`** (new) — `trait Fs` + `RealFs` + `FakeFs` |

## Purpose

`fs` is the **single filesystem dependency-injection seam** for the whole editor. The entire on-disk surface is
collapsed into **one object-safe async trait**, `pub trait Fs: Send + Sync` (`fs.rs:97`), always handled as
`Arc<dyn Fs>` and installed as a gpui `Global` (`<dyn Fs>::global(cx)` / `set_global`, `fs.rs:251`). Worktree,
project, buffer-store, and search **never call `std::fs` directly** — they take `Arc<dyn Fs>`. Exactly **two**
implementations ship: `RealFs` (production) and `FakeFs` (in-memory, test-only). That one seam is what makes the
otherwise-untestable IO layers above it deterministically testable.

The crate solves three problems at once: (1) a uniform async API over platform syscalls; (2) a **debounced,
cross-platform file-watching** primitive (`watch → (Stream<Vec<PathEvent>>, Arc<dyn Watcher>)`); (3) a
**scriptable in-memory fake** that lets a scanner/search be driven event-by-event with no disk and no races.

## Key types, modules & public API

**`src/fs.rs`** (3.4k lines) — the trait, both impls, and the value types.

- **`trait Fs`** (`:97`, `#[async_trait]`) — ~40 methods across five groups:
  - *mutations*: `create_dir`, `create_symlink`, `create_file(CreateOptions)`, `create_file_with(AsyncRead)`,
    `extract_tar_file`, `copy_file(CopyOptions)`, `rename(RenameOptions)`, `remove_dir(RemoveOptions)`,
    `remove_file`, and the trash pair `trash → TrashedEntry` / `restore(TrashedEntry)`.
  - *reads*: `load` (default method = `String::from_utf8(load_bytes)`), `load_bytes`, `open_sync ->
    Box<dyn io::Read>`, `open_handle -> Arc<dyn FileHandle>`, `read_link`, `read_dir -> Stream<Result<PathBuf>>`.
  - *writes*: `atomic_write(PathBuf, String)`, `save(&Rope, LineEnding)`, `write(&[u8])`.
  - *queries*: `metadata -> Option<Metadata>`, `canonicalize`, `is_file`/`is_dir`, `is_case_sensitive`.
  - *git + watch + misc*: `open_repo -> Arc<dyn GitRepository>`, `git_init`/`git_clone`/`git_config` (all shell
    out to the `git` binary), `watch(&Path, latency) -> (Stream<Vec<PathEvent>>, Arc<dyn Watcher>)`, `is_fake`,
    `subscribe_to_jobs -> JobEventReceiver`, and `#[cfg(test-support)] as_fake`.
- **Option structs** (`:264`–`:287`) — `CreateOptions{overwrite, ignore_if_exists}`, `CopyOptions`,
  `RenameOptions{…, create_parents}`, `RemoveOptions{recursive, ignore_if_not_exists}` — all `Copy + Default`.
- **`struct Metadata`** (`:290`) — `inode, mtime: MTime, is_symlink, is_dir, len, is_fifo, is_executable,
  is_writable`.
- **`struct MTime(SystemTime)`** (`:309`) — a newtype that **deliberately refuses `Ord`/`PartialOrd`/arithmetic**
  (doc cites "mtime comparison considered harmful"): exposes only `from_seconds_and_nanos` /
  `to_seconds_and_nanos_for_persistence` / `timestamp_for_user` and an explicitly-named escape hatch
  `bad_is_greater_than`. Prevents a whole class of "file looks unchanged" bugs.
- **`trait Watcher`** (`:71`) — `add(&Path)` / `remove(&Path)`; the returned handle's `Drop` unregisters.
  **`enum PathEventKind{Removed, Created, Changed, Rescan}`** + **`struct PathEvent{path, kind}`** (`:77`–`:85`).
- **`struct TrashedEntry`** (`:189`) + `enum TrashRestoreError` — Zed's own trash record (insulated from the
  `trash` crate's `TrashItem`) so trash/restore is roundtrippable and testable.
- **`trait FileHandle`** (`:407`) — `current_path()` follows a file across renames via per-OS fd tricks
  (`F_GETPATH` on macOS, `/proc/self/fd` on Linux, `F_KINFO` on FreeBSD, `GetFinalPathNameByHandleW` on Windows).
- **Jobs**: `JobInfo`/`JobEvent{Started, Completed}` + a `JobTracker` whose `Drop` emits `Completed` — a tiny
  progress bus for long ops (git clone). `subscribe_to_jobs()` returns the receiver.
- **`struct RealFs`** (`:399`) — production impl over `smol::fs` + `std::fs`, offloaded onto a gpui
  `BackgroundExecutor`. `watch()` (`:1059`) builds an `fs_watcher::FsWatcher`, follows symlink targets and their
  parents (skipping poll-watched virtual mounts), and returns a stream that **coalesces on the `latency` timer**
  (`executor.timer(latency).await` then drains `pending_paths`). `atomic_write` writes a `tempfile` **in the
  destination directory** then renames (avoids cross-volume EXDEV). `is_case_sensitive` probes by creating two
  case-variant temp files and caches the answer in an `AtomicU8`.
- **`struct FakeFs`** (`:1315`, `#[cfg(feature = "test-support")]`) — **the testability crown jewel.** An
  in-memory tree behind an **unfair `parking_lot::Mutex`** (chosen *for* determinism). `FakeFsState` (`:1323`)
  holds a recursive `enum FakeFsEntry{File{inode, mtime, len, content, git_dir_path}, Dir{…, entries:
  BTreeMap<String,_>, git_repo_state}, Symlink{target}}` (`:1375`) — `BTreeMap` children ⇒ **deterministic
  ordering**; monotonic `next_inode`/`next_mtime` (never the wall clock). Public helpers (`:1626`+):
  - *authoring*: `insert_tree(json)`, `insert_tree_from_real_fs`, `insert_file`, `insert_symlink`, `touch_path`.
  - *event scripting*: `pause_events`, `unpause_events_and_flush`, `flush_events(n)`, `buffered_event_count`,
    `emit_fs_event(path, kind)`, `create_file_before_next_watch_add` (races a create against a `watch.add`).
  - *introspection counters*: `read_dir_call_count`, `metadata_call_count`, `write_count_for_path`,
    `watched_paths`, `paths`/`directories`/`files`/`files_with_contents`, `trash_entries`.
  - *fake git*: `set_status_for_repo`, `set_head_and_index_for_repo`, `set_branch_name`, `insert_branches`,
    `set_blame_for_repo`, … (drives `fake_git_repo.rs`).
  - *failure injection*: `set_remove_dir_error`, `set_error_message_for_index_write`, `set_create_worktree_error`.
- **`fs_watcher.rs`** (1.7k lines) — the **`notify`** backend (Zed uses a fork; the fs Cargo.toml pins
  `notify = "9.0.0-rc.4"`). `struct FsWatcher` (`:23`) implements `Watcher`; a `WatchBackend` trait (`:792`,
  blanket-impl'd for `notify::Watcher`) abstracts native vs poll. `requires_poll_watcher(&Path)` (`:191`) sniffs
  `statfs` magic on Linux (`detect_requires_poll_watcher_linux`, `:258` — NFS/CIFS/FUSE/9P/virtiofs) and detects
  WSL `drvfs`, falling back to a poll watcher where native events silently drop. Normalizes
  `notify::EventKind → PathEventKind`, coalesces rescans (`coalesce_pending_rescans`), and case-folds paths on
  case-insensitive filesystems.
- **`fake_git_repo.rs`** (1.6k lines) — a full in-memory `GitRepository` (`FakeGitRepositoryState`: status,
  branches, index, HEAD, blame, commit graph) so git-driven worktree refresh is deterministically testable.
  Notably **git is a *process*, not a linked lib** — there is no `git2`/`gix` anywhere; `RealGitRepository`
  shells out to `git`.

## Depends on (internal, Zed-GPL)

- `gpui` — `Global`, `App`, `BackgroundExecutor`, `SharedString`, `Task` (the async runtime this crate offloads onto).
- `git` — `GitRepository`/`RealGitRepository` returned by `open_repo`.
- `rope`/`text` — `Rope` + `LineEnding` for `save`.
- `paths`/`util`/`collections`/`proto` — path sanitization, `new_command`, containers, `Timestamp` conversion.

## Used by (internal)

~56 workspace crates — worktree, project (+ every store), search, editor, and much of the app. This is the
widest IO seam in the codebase; anything that touches disk goes through `Arc<dyn Fs>`.

## Marley today · gap

Marley has **no fs abstraction.** `marley_project::list_files_in` walks `std::fs` directly (the crate doc even
notes the walk "climbs to `/`" because it is unmocked); the app reads/writes files inline; tests use real
`tempfile` trees. Consequences: (1) the file tree / finder / any future search can't be tested deterministically
(every test hits disk); (2) there's no seam to ever slot a remote/overlay fs behind; (3) mtime/case/symlink
edge-cases are handled ad hoc. This is the **biggest foundational gap** in the whole subsystem, and every
higher layer's testability depends on closing it.

## Reimplementation on our stack — `marley_fs`  `[Marley-original]`

The **highest-leverage foundational move.** Introduce a `marley_fs` crate with a **narrow** `trait Fs`
(`async-trait`, used as `Arc<dyn Fs>`), sized to Marley rather than Zed's full 40-method surface:

- **Phase 1 — trait + two impls, no watching.** `load` / `save` / `atomic_write` / `metadata` / `read_dir` /
  `canonicalize` / `rename` / `remove` / `create_dir`. Ship `RealFs` (over `smol` + the gpui
  `background_executor` Marley already has) and `FakeFs` behind a `test-support` feature (in-memory `BTreeMap`
  tree, monotonic inode/mtime). Adopt the **`MTime`-without-`Ord`** guard verbatim as a design pattern — it
  prevents real bugs and costs nothing. Retrofit `list_files_in` to take `&dyn Fs` and convert its tests to
  `FakeFs`. **This immediately pays off** as the test substrate for `marley_worktree`.
- **Phase 2 — `watch()`.** Add `watch(&Path, latency) -> (Stream<Vec<PathEvent>>, Arc<dyn Watcher>)` over the
  **`notify`** crate with the `requires_poll_watcher` poll fallback and the `latency`-timer coalescing. This is
  the prerequisite for a live worktree.
- **Keep git out of scope initially.** Marley already gets git context from the shell (`terminal_blocks`); add
  `open_repo` only if a non-shell git view later arrives. Skip trash/tar/`FileHandle` until a caller needs them.

**Why this fits Marley's cov/MSI-100 discipline exactly.** A live `RealFs::watch` is inherently a
coverage-excluded, self-test-only shim (real syscalls). But `FakeFs` **is** the mechanism that makes the *logic*
above it testable: the scanner/search take `Arc<dyn Fs>`, a test builds an exact tree, single-steps
`flush_events(n)` on gpui's deterministic executor, and asserts. Keep `RealFs`'s thin syscall wrappers as the
**only** `#[mutants::skip]` shim; put every decision on the fake-driven side so pure seams hit the strict gate.

### Sequencing
1. `trait Fs` + `RealFs` + `FakeFs` (no watch) — retrofit `list_files_in(&dyn Fs)`, tests → `FakeFs`.
2. `watch()` (notify + poll fallback) + `PathEvent`/`Watcher` — unblocks the live worktree.

## Provenance

- `[permissive/public]` — **`notify`** (MIT/Apache) for cross-platform watching; **`ignore`** (Unlicense/MIT);
  `smol`/`async-trait`/`futures`; the system `git` binary; `trash`/`ashpd`. All directly usable under Marley's
  own license.
- `[Zed-derived]` — the **design patterns** (freely reimplementable, clean-room, *not* copied): one async `Fs`
  trait used as an installed `Global`; the `RealFs`/`FakeFs` DI split; the scriptable-fake testability pattern
  (`pause`/`flush_events(n)`/`emit_fs_event` + introspection counters); `watch → (Stream<Vec<PathEvent>>,
  Watcher-with-Drop)` with `latency` coalescing; atomic-write-via-same-dir-tempfile; the `MTime`-without-`Ord`
  guard.
- `[Marley-original]` — the down-scoped `marley_fs` crate and the FakeFs-as-cov/MSI-substrate strategy.

## Notes / gotchas

- **`MTime` refuses `Ord` on purpose** — never sort or `<`-compare mtimes to decide dirtiness. Carry this guard
  into `marley_fs`.
- **`FakeFs` uses an *unfair* mutex intentionally** — fairness would reintroduce scheduling nondeterminism the
  fake exists to eliminate.
- **Watching virtual filesystems needs the poll fallback** — NFS/CIFS/FUSE/9P(WSL)/virtiofs silently drop
  native events; `requires_poll_watcher` is the guard. Don't skip it if Marley ever opens network mounts.
- **Git is a subprocess, not a crate** — same shell-out posture Marley already uses; no `git2`/`gix` to license-audit.
- **Remote fs is *not* an `Fs` impl** — Zed's remote file access is a proto request to a headless server, not a
  decorator over the trait. If Marley ever wants remote files, a remote-`Fs` impl is the clean, seam-preserving
  way (§5 of subsystem 06) — the trait is decorator-friendly even though Zed ships no wrapper.
