# Subsystem 06 — Project, FS, Worktree & Search

Part of the Zed architecture reference for **Marley** (round 1). Zed is the EDITOR reference for Marley's
editing surface; this doc deconstructs the **file-model + orchestration + search** spine so Marley can
reimplement each capability in its own code.

> **Provenance is load-bearing here.** Zed is **GPL-3.0**; its *project / worktree / snapshot / multibuffer /
> headless-remote* designs are **`[Zed-derived]`** and belong only in Marley's (planned) GPL editor/terminal
> layer — never the brain. But almost all the *hard machinery underneath* is **`[permissive/public]`**: fs
> watching is the **`notify`** crate (MIT/Apache), gitignore parsing is the **`ignore`** crate (Unlicense/MIT),
> globbing is **`globset`/`wax`**, and content search is **`regex` / `aho-corasick` / `fancy-regex`** — all
> permissively licensed and directly usable. Zed's own contribution is the *architecture* that composes them
> (the immutable-snapshot worktree, the host-agnostic Project hub, the editable-multibuffer results). Marley's
> reimplementation on its own stack is **`[Marley-original]`**. Each section tags the split.

---

## TL;DR — the layer cake

Zed builds file/search capability as a strict dependency stack; every layer is swappable at its lower seam:

```
   crates/search        editable find/replace UI  → reuses the generic Editor + MultiBuffer  [Zed-derived]
   ────────────────────────────────────────────────────────────────────────────────────────
   crates/project       THE HUB: Project = façade owning ~15 sub-stores (worktree/buffer/    [Zed-derived]
                        lsp/git/search/tasks/terminals…), host-agnostic (local|ssh|collab)
      ├ worktree_store  holds N worktrees;  buffer_store  holds open buffers;  project_search  the engine
   ────────────────────────────────────────────────────────────────────────────────────────
   crates/worktree      one root's live file tree: immutable Snapshot over a SumTree<Entry>, [Zed-derived]
                        kept current by a BackgroundScanner reacting to fs events + gitignore  (ignore = [public])
   ────────────────────────────────────────────────────────────────────────────────────────
   crates/fs            Arc<dyn Fs>: ONE async trait. RealFs (std/smol + notify) │ FakeFs     [Zed-derived]
                        (in-memory, deterministic, scriptable events — the testability engine)  (notify = [public])
   ────────────────────────────────────────────────────────────────────────────────────────
   crates/remote(_server)  ALT BACKEND: a headless Zed on the far host over system-ssh stdio;  [Zed-derived]
                        the local Project mirrors worktree/buffer snapshots via protobuf RPC   (ssh binary = [public])
```

**Two ideas carry the whole subsystem.** (1) **Cheap immutable snapshots** — the worktree is a
copy-on-write B-tree so a full, consistent view can be *published on every change* and two views diffed in
O(changed); the foreground never mutates, it only receives. (2) **Host-agnostic orchestration** — every store
is `enum State { Local { fs } | Remote { proto_client, project_id } }`, so the *same* `Project` code drives a
local disk, an SSH daemon, or a collab host by swapping a type-erased RPC client. Local-first Marley can adopt
the `Local` half of everything and stub `Remote` until an intake justifies it.

### Marley baseline (post-M15) vs Zed — the gap at a glance

| Capability | Marley today | Zed | Verdict |
|---|---|---|---|
| **fs access** | direct `std::fs` calls scattered in app + `marley_project`; `tempfile` in tests | `Arc<dyn Fs>` trait, `RealFs` + `FakeFs` | **no abstraction** — biggest foundational gap |
| **file tree** | `marley_project::FileTree`, built **once** at boot from a one-shot `list_files_in` walk (cap 10k) | live `Worktree` `Snapshot`, incrementally updated | **static, no watching** |
| **ignore handling** | hardcoded `should_skip` (`.git`/`node_modules`/`target`/`.DS_Store`) | `.gitignore` parsed via `ignore` crate, hierarchical `IgnoreStack`, `file_scan_exclusions` globs | **no real gitignore** |
| **the "project"** | `Project { root, name, is_git }` — a root descriptor; orchestration lives in `marley_app::RootView` | `Project` = hub owning ~15 stores | Marley's Project ≈ Zed's *Worktree*, not Zed's *Project* |
| **fuzzy file open** | `⌘P` finder over `list_files_in` via `nucleo` (`marley_search_core`) | file finder over worktree entries' `char_bag` | **present, comparable** |
| **in-file find** | `find.rs::find_matches(haystack, query) -> Vec<Range>` — literal substring, one buffer | full `BufferSearchBar`: regex, whole-word, replace, select-all | **basic; no regex/replace** |
| **project-wide find/replace** | **none** (`search_everything` fuzzy-matches file *names*, sessions, actions — never contents) | streaming search across fs + open buffers → **editable multibuffer** | **entirely absent** — flagship gap |
| **remote** | `marley_remote`: spawn system `ssh` as a **terminal pane** (no secrets, argv-safe) | headless server + thin client + proto-mirrored remote worktree/LSP | **fundamentally different scope** |

---

## §1 — The `fs` abstraction (real vs remote vs fake)  ·  crate `fs`

### Design  `[Zed-derived]` (`notify` = `[permissive/public]`)

The entire filesystem surface is **one object-safe async trait**, `pub trait Fs: Send + Sync`
(`fs/src/fs.rs:97`), always used as `Arc<dyn Fs>` and installed as a gpui `Global` (`dyn Fs::global(cx)`). It is
the single dependency-injection seam of the whole subsystem — worktree, project, and search all take
`Arc<dyn Fs>`, never touch `std::fs` directly. Exactly **two** implementations exist in the repo.

- **Method surface** (~40 async methods): mutations (`create_dir`, `create_file(CreateOptions)`, `rename`,
  `copy_file`, `remove_file`, `trash`/`restore`), reads (`load`/`load_bytes`, `read_dir -> Stream<PathBuf>`,
  `open_handle -> Arc<dyn FileHandle>`, `read_link`), writes (`atomic_write`, `save(&Rope, LineEnding)`),
  queries (`metadata -> Option<Metadata>`, `canonicalize`, `is_case_sensitive`), git (`open_repo -> Arc<dyn
  GitRepository>`, `git_init`/`git_clone`), and the critical `watch(&Path, latency) -> (Stream<Vec<PathEvent>>,
  Arc<dyn Watcher>)`.
- **`RealFs`** (`fs.rs:399`) — production, over `smol::fs` + `std::fs` offloaded onto the gpui
  `BackgroundExecutor`. `watch()` returns a debounced batch stream (the `latency` arg is a coalescing window)
  plus an `Arc<dyn Watcher>` whose `Drop` unregisters. `atomic_write` = write a `tempfile` **in the destination
  directory** then rename (avoids cross-volume rename errors). `MTime` is a newtype that **deliberately refuses
  `Ord`** to prevent mtime-comparison bugs.
- **`FakeFs`** (`fs.rs:1315`, `#[cfg(feature = "test-support")]`) — **the testability crown jewel.** An
  in-memory tree (`BTreeMap` children ⇒ deterministic ordering) behind an *unfair* `parking_lot::Mutex` (for
  determinism), with **monotonic fake inodes/mtimes** (never the wall clock), **scriptable watcher events**
  (`pause_events`/`flush_events(n)`/`emit_fs_event`), test-authoring helpers (`insert_tree(json)`,
  `insert_tree_from_real_fs`), introspection counters (`metadata_call_count`, `read_dir_call_count`), and
  failure injection. Because worktree/project/search all take `Arc<dyn Fs>`, a test builds an exact tree, drives
  the *same* API the product uses, single-steps event delivery, and asserts — no disk, no races, on gpui's
  deterministic executor.
- **`fs_watcher.rs`** — backend is the **`notify` crate (v9)**: FSEvents (macOS) / inotify (Linux) /
  ReadDirectoryChanges (Windows) / kqueue (BSD), abstracted behind a `WatchBackend` trait. A **single global
  multiplexer** (`GlobalWatcher`, one native + one poll watcher shared by all `FsWatcher`s) plus a **polling
  fallback** for network/virtual filesystems (`requires_poll_watcher` sniffs `statfs` magic — NFS, WSL drvfs).
  Events are normalized `notify::EventKind → PathEventKind{Created,Changed,Removed,Rescan}`, coalesced, and
  case-/NFC-folded on case-insensitive macOS.
- **Git = external process, not a linked lib.** `RealGitRepository` shells out to the `git` binary; there is
  **no `git2`/`gix`** anywhere. `fake_git_repo.rs` simulates the whole `GitRepository` trait (status/branches/
  blame/stage/commit) so git-driven refresh is deterministically testable too.
- **Remote fs is NOT here.** Only `RealFs` + `FakeFs` implement `Fs`; remote file access is a *proto request to
  the headless server* (§5), not an `Fs` impl. The trait is decorator-friendly but no wrapper ships.

### Marley today  ·  gap

Marley has **no fs abstraction**. `marley_project::list_files_in` walks `std::fs` directly; the app reads/writes
files inline; tests use real `tempfile` trees (the `marley_project` doc even notes the walk "climbs to `/`"
because it is unmocked). Consequences: (1) no way to test the file tree / finder / future search
*deterministically* — every test hits disk; (2) no seam to ever slot a remote or overlay fs behind; (3)
mtime/case/symlink edge cases are handled ad hoc.

### Reimplementation on our stack — `marley_fs`  `[Marley-original]`

Highest-leverage foundational move; unlocks testability for everything above it. Introduce a **`marley_fs`**
crate with a narrow `trait Fs` (async via `async-trait`, `Arc<dyn Fs>`) — a *subset* of Zed's surface sized to
Marley: `load`/`save`/`atomic_write`/`metadata`/`read_dir`/`canonicalize`/`rename`/`remove`/`watch`. Ship
`RealFs` (over `smol` + gpui's `background_executor`, which Marley already has) and a **`FakeFs`** behind a
`test-support` feature (in-memory `BTreeMap` tree, monotonic inode/mtime, pausable scriptable events). Adopt the
**`notify` crate** for `RealFs::watch` with a poll fallback, and the `MTime`-without-`Ord` guard.

- **Fit with Marley's discipline:** this crate is *exactly* how Marley reaches "pure seams cov/MSI 100" for the
  file layer. Today a live scanner would be a coverage-excluded, self-test-only shim; with `FakeFs` the scanner
  becomes deterministic and unit-testable (drive events, assert snapshot) — the fake *is* the mechanism that
  lets otherwise-untestable IO logic hit the strict gate. Keep `RealFs`'s thin syscall wrappers as the only
  shim; put all decision logic on the fake-driven side.
- Keep git out of scope initially (Marley already has `terminal_blocks` git context from the shell); add
  `open_repo` later only if a non-shell git status view arrives.

### Sequencing
1. `marley_fs` = `trait Fs` + `RealFs` + `FakeFs` (no watching yet) — retro-fit `list_files_in` to take
   `&dyn Fs`; convert its tests to `FakeFs`. **Immediately pays off** as the test substrate for §2.
2. Add `watch()` (notify + poll fallback) + `PathEvent`. Prerequisite for the live worktree.

### Provenance
`[permissive/public]` — `notify` (MIT/Apache) for watching; `smol`/`async-trait`/`futures`; the `git` CLI.
`[Zed-derived]` — the *shape* (one async `Fs` trait + `RealFs`/`FakeFs` DI split, scriptable-fake testability
pattern, `watch → (Stream<Vec<PathEvent>>, Watcher-with-Drop)`, atomic-write-via-same-dir-tempfile). These are
design patterns, not code — reimplement freely under Marley's own license, clean-room from Zed source.

---

## §2 — The `worktree` model (file tree, fs watching, ignore)  ·  crate `worktree`

### Design  `[Zed-derived]` (`ignore` crate = `[permissive/public]`)

A `Worktree` is an **in-memory, immutable-snapshot mirror of one filesystem subtree** (one opened root folder),
kept current by a background scanner reacting to fs events. `pub enum Worktree { Local(LocalWorktree),
Remote(RemoteWorktree) }` (`worktree.rs:95`) — both `Deref` to a shared `Snapshot`.

- **`Snapshot`** (`worktree.rs:176`) — the shared immutable core: **`entries_by_path: SumTree<Entry>`** and
  **`entries_by_id: SumTree<PathEntry>`** (two indexes over the same files) + `scan_id`/`completed_scan_id`
  counters. `SumTree` is Zed's own copy-on-write B+-tree where each node caches a composable `Summary` of its
  subtree — so you get ordered/prefix traversal, O(log n) aggregate queries ("non-ignored file count under this
  dir"), stable-id lookup, **and O(1)-ish `.clone()`** (Arc bumps). That last property is the linchpin: every
  change publishes a *whole fresh snapshot* cheaply, and the foreground diffs old-vs-new in O(changed).
- **`Entry`** (`worktree.rs:3795`) — the per-path record: `id: ProjectEntryId` (stable handle across renames),
  `kind: EntryKind{UnloadedDir,PendingDir,Dir,File}`, `path: Arc<RelPath>`, `inode`, `mtime`, `canonical_path`
  (for symlinks), `size`, `char_bag: CharBag` (lowercased chars — fuels fuzzy file-open), and status booleans
  `is_ignored` / `is_hidden` / `is_private` (.env) / `is_external` (symlink out of tree) / `is_always_included`.
  **There is NO `git_status` field** — git status is fully decoupled (computed by a separate `GitStore`); the
  worktree only tracks *repo locations*. Copy this: do not put `git_status` on your entry.
- **`BackgroundScanner`** (`worktree.rs:4166`) — owns all mutation on a gpui background task; phases
  `InitialScan → EventsReceivedDuringInitialScan → Events`. Initial scan fans `read_dir` jobs across the
  executor via a job channel (forcing `.git`/`.gitignore` to the front so ignore state is known before siblings
  are classified). Steady state is a **`select_biased!` loop** (explicit refresh requests beat fs-event noise)
  that coalesces bursts, then `reload_entries_for_paths` recomputes just the affected entries. `.git`-internal
  churn (`*.lock`, `logs/`, `objects/`) is filtered to avoid needless rescans. **Publishing** = clone snapshot,
  diff against `prev_snapshot`, send `ScanState::Updated{snapshot, changes}` over a channel; the foreground
  installs it and emits `UpdatedEntries`. **Lazy expansion:** ignored/hidden/external dirs are inserted as
  `UnloadedDir` placeholders and only scanned when expanded — keeps huge trees cheap.
- **Ignore handling** (`ignore.rs`) — wraps the **`ignore` crate's `Gitignore`** (the ripgrep-ecosystem parser).
  `IgnoreStack` is a persistent linked list built top-down as the scanner descends (each dir cheaply shares its
  parent's stack via `Arc`); `is_abs_path_ignored` walks deepest-first, first-match-wins, honoring negation
  (`!pat`) and git precedence (dir-local `.gitignore` → `.git/info/exclude` → global). Layered on top,
  `WorktreeSettings` (`worktree_settings.rs`) holds `PathMatcher` globs: `file_scan_exclusions` (never scan —
  beats everything), `file_scan_inclusions` (scan even if gitignored), `private_files`, `hidden_files`.
- **Remote worktree never touches a filesystem** — it materializes `proto::UpdateWorktree` diffs into the same
  `Snapshot` type via a double-buffered `Arc<Mutex<(Snapshot, Vec<Update>)>>` (wire-ingest decoupled from
  model-apply). Same snapshot, two producers (fs scanner vs proto stream). This is what §5 mirrors.

### Marley today  ·  gap

`marley_project::FileTree` is a **static, one-shot** structure: `Project::discover_in` finds the nearest `.git`
root, `list_files_in` does a single capped (10k) `std::fs` stack walk pruning a hardcoded `should_skip` set,
`FileTree::from_files` nests it, and `RootView` builds it **once at boot** (per the crate doc). Gaps vs Zed:
**(a)** no fs watching — create/delete/rename on disk is invisible until relaunch; **(b)** no real `.gitignore`
(only the 4-name `should_skip`); **(c)** no stable entry ids, no incremental diff, no lazy expansion, no
per-entry metadata (mtime/size/ignored); **(d)** the tree and the `⌘P` finder each re-walk independently.

### Reimplementation on our stack — `marley_worktree`  `[Marley-original]`

A `marley_worktree` crate that turns the static tree into a **live snapshot**. Marley does *not* need Zed's full
weight (no collab, no per-entry git status), but should adopt the core moves:

- **A snapshot over an ordered map.** Marley may start with a simpler `BTreeMap<RelPath, Entry>` behind an `Arc`
  (clone-to-publish) rather than a bespoke SumTree — the summary/aggregate queries (file counts) are a *later*
  optimization; the essential property is *cheap immutable publish + diff*. Keep `Entry{id, kind, path, mtime,
  is_ignored, char_bag}` and stable `EntryId`s so the finder, tree UI, and future search share one identity.
- **A scanner** over `marley_fs`: initial parallel walk + a `watch()`-driven steady loop that
  `reload_entries_for_paths`. Publish snapshots on a channel; the gpui `RootView` swaps the `Arc` and re-renders
  (reusing today's `visible_rows`/`toggle`, now fed by a live snapshot instead of a boot-time `Vec`).
- **Real ignore** via the **`ignore` crate** + an `IgnoreStack`, replacing `should_skip`. Add a
  `file_scan_exclusions`-style glob setting (Marley already has a TOML settings framework, `marley_settings`).
- **Fold `list_files_in` + the `⌘P` finder into the worktree** — the finder ranks `snapshot.files()` by
  `char_bag`/`nucleo` instead of re-walking. One source of truth for "the project's files."

### Sequencing
1. `Entry` + `Snapshot` (immutable, clone-to-publish) + port `from_files`/`visible_rows` onto it. Pure, cov/MSI
   100, `FakeFs`-driven.
2. `BackgroundScanner` (initial scan only) over `marley_fs` — replace the boot-time walk. Finder reads
   `snapshot.files()`.
3. Wire `fs.watch()` → incremental `reload_entries_for_paths` → live tree (create/delete/rename reflected).
4. Swap `should_skip` → `ignore` crate + `IgnoreStack` + a `file_scan_exclusions` setting.

### Provenance
`[permissive/public]` — `ignore` (gitignore parsing, Unlicense/MIT), `globset` (PathMatcher). `[Zed-derived]` —
the immutable-snapshot-published-on-every-change model, the `Entry` field taxonomy (incl. *git-status-decoupled*),
the scanner phases + lazy-expansion, the hierarchical `IgnoreStack` composition. `[Marley-original]` — the
reimplementation, and the choice to start on a plain ordered map before a SumTree. (SumTree itself is a
summarizing COW B-tree — the *concept* is CS literature/public; Zed's *implementation* is GPL, so reimplement,
don't lift.)

---

## §3 — The `project` model (the orchestration hub)  ·  crate `project`

### Design  `[Zed-derived]`

`pub struct Project` (`project.rs:213`) is a gpui `Entity` that **owns almost no domain data** — it is a **thin
façade + event router** holding ~15 sub-store `Entity` handles and wiring them together: `worktree_store`,
`buffer_store`, `lsp_store`, `git_store`, plus `dap_store`, `task_store`, `image_store`, `toolchain_store`,
`context_server_store`, `terminals`, `environment`, `settings_observer`, three `SearchHistory`, etc. Public
methods are one-line delegations (`open_buffer` → `buffer_store.update(…)`); the value is in two cross-cutting
layers:

- **Host-agnostic by construction (the central idea).** Two orthogonal axes encode three modes: an
  `enum ProjectClientState { Local, Shared{remote_id}, Collab{replica_id, capability, …} }` and an
  `Option<Entity<RemoteClient>>`. Predicates: `is_local()`, `is_via_remote_server()` (ssh — i.e.
  `remote_client.is_some()`), `is_via_collab()`. Every *store* carries its own
  `enum State { Local{fs} | Remote{upstream_client: AnyProtoClient, project_id, path_style} }`. Because
  `AnyProtoClient` is a type-erased protobuf RPC client with a uniform request/send/subscribe surface, the store
  *logic is identical* whether the peer is absent (local fs), an SSH daemon, or a collab host — you swap the
  client, not the code. `Project::local()` / `Project::remote(ssh)` / `Project::in_room(collab)` differ only in
  which client each store is handed.
- **Event translation + proto mirroring.** Each store `impl EventEmitter<StoreEvent>`; the Project
  `cx.subscribe`s to all of them and fans their narrow events (`WorktreeStoreEvent::WorktreeUpdatedEntries`,
  `BufferStoreEvent::BufferAdded`…) into **one workspace-facing `project::Event` stream**, while
  simultaneously forwarding mutations onto the proto wire (so a host can serve guests). `WorktreeStore` holds
  `worktrees: Vec<WorktreeHandle>` and `create_worktree` dispatches local-vs-remote; `BufferStore` keys open
  buffers by `BufferId` with two indexes (`path_to_buffer_id`, `local_buffer_ids_by_entry_id`) that tie a buffer
  back to a worktree `Entry`. Concurrent opens are coalesced through `loading_worktrees`/`loading_buffers`
  (`Shared<Task>`).

### ★ How this relates to Marley's Workspace → Project → Tab

This is the pivotal architectural question, and the mapping is not 1:1:

| Zed | Role | Marley equivalent |
|---|---|---|
| **`Workspace`** (separate `workspace` crate) | the **window**: `PaneGroup` + `panes` + docks, holds **one** `Entity<Project>` | Marley's `RootView` / the Workspace→Project→Tab shell in `marley_app` |
| **`Project`** | the **backend hub**: owns **many** worktrees + buffers + LSP + search + host-abstraction | **has no equivalent** — Marley has no hub |
| **`Worktree`** | **one** opened root folder + its live file tree | **Marley's `Project { root, name, is_git }`** |

So **Marley's "Project" is really Zed's "Worktree"** (a single root + tree), and Marley's orchestration
currently lives ad hoc in `RootView`. Zed's `Project` layer — a hub owning *N* worktrees behind a host-agnostic
seam — has **no counterpart** in Marley.

**Does a terminal-first tool need the hub? Recommendation: not yet — but it is exactly where chad's
workspace-centric direction lands.** Today Marley opens one root, and `RootView` coordinating a tree + a finder
+ terminal panes is sufficient; importing Zed's full `Project` (collab, LSP, DAP, proto) would be enormous
dead weight. **However**, three forces pull a lightweight hub into existence, and two are already on Marley's
roadmap:
1. **Multi-workspace / launcher (chad's stated M13+ direction — "multiple workspaces open at once, like
   PhpStorm; the side rail highlights the active one").** That is *precisely* Zed's `Workspace(window) →
   Project(hub) → Worktree(folder)` shape. A per-workspace holder that owns its worktree(s) + open buffers is
   the natural home for "which folders are open in this workspace," and the rail highlight is a `ProjectEvent`
   subscription.
2. **Project-wide search (§4)** needs *someone* to own "all worktrees + all open buffers" to fan a query across
   — that owner is a proto-hub-shaped object.
3. **Editor-as-peer (the M13 intake)** means buffers are shared between the tree, the finder, the search
   results, and terminal cwd — a `BufferStore`-like open-buffer registry (keyed by path, tied to entries)
   prevents three subsystems opening three copies of one file.

**The move:** grow a **minimal `marley_project` hub incrementally** — a holder for `Vec<Worktree>` + an
open-buffer registry + a single `ProjectEvent` bus — implementing **only the `Local` arm** of the per-store
`State` enum, and *keeping the `Remote{proto}` seam as an unimplemented hook* so §5 can slot in later without
touching call sites. Do **not** import collab/LSP/DAP. This is the project-hub agent's "adopt the Local half,
stub Remote" guidance, scoped to Marley.

### Marley today  ·  gap

`Project` is a value struct (`{root, name, is_git}` + file-listing helpers), not an entity, not a hub. There is
no open-buffer registry (the editor holds one `Buffer` at a time), no event bus unifying tree/finder/terminal,
and no seam for multiple roots or a remote backend. Gap = **the entire orchestration + host-abstraction layer**.

### Reimplementation  `[Marley-original]`  ·  Sequencing
1. Promote `marley_project` to a gpui `Entity` hub owning `worktrees: Vec<Worktree>` + an
   `open_buffers: HashMap<Path, BufferHandle>` registry; emit a `ProjectEvent` the `RootView` subscribes to.
   Local-only.
2. Route the future project-search (§4) and the finder through the hub's worktrees.
3. When multi-workspace ships: one hub per workspace; the launcher/rail iterate hubs. (Keep the `Remote` seam
   dormant for §5.)

### Provenance
`[Zed-derived]` — the façade+router pattern, the host-agnostic per-store `State{Local|Remote}` + `AnyProtoClient`
seam, the buffer↔entry indexing, `Workspace(window) → Project(hub) → Worktree(folder)` layering. `[permissive/
public]` — gpui (Apache-2.0; Marley already depends on `gpui = 0.2.2`) supplies the `Entity`/`Context`/
`EventEmitter`/`BackgroundExecutor` primitives directly, no reimplementation needed. `[Marley-original]` — the
scoped Local-only hub.

---

## §4 — Project-wide find/replace (search across files → editable results)  ·  crates `project` + `search`

The flagship gap. Zed splits it: **`crates/project`** = a headless, UI-free search *engine*; **`crates/search`**
= the UI that renders results as an **editable multibuffer**. The single highest-leverage design decision is
that results are a *generic editor over a multibuffer*, so edit/save/replace-all come "for free."

### Design — the query model  `[Zed-derived]` (algorithms = `[permissive/public]`)

`enum SearchQuery` (`project/search.rs:65`) — two variants sharing a common `inner: SearchInputs`:
- **`Text{ search: AhoCorasick, replacement, whole_word, case_sensitive, include_ignored, inner }`** — literal
  search via the **`aho-corasick`** crate (streams over the buffer's `Rope` bytes without allocating a String).
- **`Regex{ regex: fancy_regex::Regex, multiline, one_match_per_line, … }`** — via **`fancy-regex`**
  (backtracking; supports lookaround/backrefs). Case-insensitive non-ASCII text *falls back* from aho-corasick
  to regex (aho-corasick can't Unicode-case-fold).
- **`SearchInputs{ query, files_to_include: PathMatcher, files_to_exclude: PathMatcher, match_full_paths,
  buffers: Option<Vec<Entity<Buffer>>> }`** — include/exclude are compiled globs (**`wax`/`globset`**);
  `buffers: Some(_)` scopes to "open buffers only."
- **`enum SearchResult`** is a *stream* with control signals: `Buffer{ buffer: Entity<Buffer>, ranges:
  Vec<Range<Anchor>> }` + `Searching`/`WaitingForScan`/`LimitReached`. **Matches are `Anchor` ranges bound to a
  live `Buffer`** — not raw file offsets — which is *why* results are inherently editable (§ results UI).

### Design — the execution pipeline  `[Zed-derived]`

`Project::search_impl` picks `Search::local | remote | open_buffers_only` and returns a `#[must_use]`
`SearchResultsHandle`. **It is NOT ripgrep** — a hand-rolled multi-stage async pipeline of `async_channel`s
fanned across the gpui `BackgroundExecutor` with `max(num_cpus-1, 1)` `Worker`s (a `select_biased!` loop that
drains expensive full-scans before cheap probes). Stages:
1. **`provide_search_paths`** — per worktree, await scan-complete, stream every candidate `Entry` (pre-scanning
   gitignored dirs first if `include_ignored`).
2. **`handle_scan_path`** — cheap glob include/exclude filter, no I/O. **Buffer-precedence branch:** if the
   entry is an open buffer, skip disk and search the *in-memory* version (dirty buffers never read stale disk).
3. **`handle_find_first_match`** — `fs.open_sync` + `query.detect()` streams for the *first* match only, and
   **rejects invalid-UTF-8 (binary) files early**. Avoids loading a `Buffer` for non-matching files.
4. **`open_buffers`** (main thread — workers can't open buffers) → **`handle_find_all_matches`** runs the full
   `query.search()` over the buffer snapshot, emitting `Vec<Range<Anchor>>`.
5. Ordering + caps: `MAX_SEARCH_RESULT_FILES = 5_000`, `MAX_SEARCH_RESULT_RANGES = 10_000` → `LimitReached`.

Remote/ssh delegation: send `proto::FindSearchCandidates` → the headless host runs *its* local search in
"matching-buffers-only" mode → streams buffer ids → client syncs those buffers → runs the full-match pass
locally.

### Design — the editable MULTIBUFFER results (the flagship)  `[Zed-derived]`

Results are **not a custom list widget** — they are a normal `Editor::for_multibuffer(...)`. A **`MultiBuffer`**
(Zed's own crate) stitches ranges from many real `Buffer`s into one coordinate space:
- An **`ExcerptRange{ context: Range, primary: Range }`** = the lines shown (`context`, match ± N lines) and the
  exact highlighted span (`primary`). Excerpts are grouped/sorted by a `PathKey{ sort_prefix, path }`.
- `ProjectSearch::consume_search_stream` drains `SearchResult`s and, per file, calls
  `multibuffer.set_anchored_excerpts_for_path(PathKey, buffer, ranges, context_lines)` — which builds+merges
  excerpt ranges off-thread and splices them into the multibuffer's excerpt `SumTree`.
- **Editability is free:** because the results pane is a generic multibuffer editor, typing in an excerpt edits
  the underlying `Buffer` in place; saving persists via the normal buffer-save path — *no propagation code in
  the search crate*. **Replace-all** = `editor.replace_all(match_ranges, query)` builds an edit list (regex:
  per-match `replacement_for` capture substitution; literal: one string) applied as **a single transaction**.
- The in-file `BufferSearchBar` shares the same `SearchableItem` trait, `SearchOptions` bitflags
  (`WHOLE_WORD|CASE_SENSITIVE|REGEX|INCLUDE_IGNORED|…`), and three-history model (query/include/exclude).

### Marley today  ·  gap

**Zero content search.** `command_bar.rs::search_everything` fuzzy-matches file *names* + session titles +
action labels via `nucleo` — never file contents. In-file find is `find.rs::find_matches(haystack, query) ->
Vec<Range<usize>>`, a literal substring scan over one buffer (no regex, no replace, no cross-file). There is
**no `MultiBuffer`** in Marley's editor (the `Buffer` is single-buffer; the "multi" in the code is *multibyte*).
Gap = **the query model, the fan-out engine, AND the multibuffer results surface** — the largest net-new build
in this subsystem.

### Reimplementation on our stack  `[Marley-original]`

Build in the same two layers, sized down:
- **Engine (`marley_project` search):** a `SearchQuery{ Text(AhoCorasick) | Regex(fancy_regex) }` +
  `SearchInputs{ include/exclude via globset }` + a streaming `SearchResult`. Reuse the pipeline shape over
  Marley's `background_executor`: worktree entries → glob filter → cheap first-match probe over `marley_fs`
  (binary-skip) → full match. **Open/dirty-buffer precedence** matters even single-user (the editor may have
  unsaved edits). Cap results. This is `[permissive/public]` algorithms composed by a `[Zed-derived]` pipeline
  design — pure, `FakeFs`-driven, cov/MSI 100-friendly.
- **Results surface — the decision point.** Zed's editability hinges on a `MultiBuffer`, which Marley's editor
  does **not** have. Two phased options:
  - **Phase 1 (no multibuffer): a read-only results panel** — a Left/Right-dock list of `file:line` hits (like
    a `grep` pane); Enter opens the file at the match in the existing single editor. Delivers 80% of the value
    (find across the project) with none of the multibuffer cost. Fits Marley's current docks + finder idioms.
  - **Phase 2 (editable): build a minimal `MultiBuffer`** in `crates/editor` — an excerpt = `(source Buffer,
    context range, primary range)`, rendered by the existing editor over a stitched coordinate space; replace-
    all as one transaction. This is a real editor investment and should be **gated on the editor-as-peer
    intake** (M13), since the same `MultiBuffer` powers diagnostics/references later.

### Sequencing
1. Search engine (query + streaming pipeline over worktree+fs+open-buffers) — headless, testable.
2. Phase-1 read-only results dock (`file:line`, Enter-to-open). **Ships project-wide find on its own.**
3. In-file find upgrade: regex + whole-word + replace on the single buffer (reuse `SearchOptions`).
4. Phase-2 `MultiBuffer` + editable results + replace-all — **only alongside the editor-as-peer milestone.**

### Provenance
`[permissive/public]` — `aho-corasick`, `fancy-regex`, `regex`, `wax`/`globset`, `nucleo` (already vendored).
`[Zed-derived]` — the 2-variant query enum, the candidate→probe→match pipeline with buffer precedence, the
`ExcerptRange{context,primary}` + `PathKey` + reuse-a-generic-editor-for-editable-results model. `[Marley-
original]` — the down-scoped engine + the phased (read-only → multibuffer) results plan.

---

## §5 — Remote worktrees (SSH / dev-containers)  ·  crates `remote` + `remote_server`

### Design  `[Zed-derived]` (system `ssh` binary = `[permissive/public]`)

Zed's remote model is **fundamentally different from a remote *shell*.** It does not run a terminal on the far
host — it runs **a second, headless copy of Zed's own engine** there (`remote_server` binary) and makes the
local Zed a thin RPC client. The remote **fs, worktree scanning, buffers, language servers, git, tasks** all
*live and execute on the server*; the client holds only **mirrored snapshots** synced over protobuf RPC. Opening
a file = a proto request the server fulfills against *its* disk; find-in-files = a search that runs *on the
server*; language intelligence = LSP servers the *server* launches. It is "collab with a party of one, where
the peer is a daemon."

- **Transport = the system `ssh` binary** (`util::command::new_command("ssh")`) — **no `russh`/`ssh2`/`libssh`
  crate anywhere** (same shell-out posture as Marley, at the process level). But it layers a full RPC stack on
  the pipe: `ControlMaster` multiplexing (one long-lived auth'd session reused, skipping re-auth), `askpass` for
  password/2FA, versioned server-binary provisioning (probe `uname`/version → download-on-server via curl or
  upload via **sftp/scp**), then launches `<remote-binary> proxy` and pumps **length-prefixed `prost`-encoded
  `Envelope`s** over ssh stdio (stderr carries JSON log records back).
- **`RemoteConnection` trait** (`remote_client.rs:1589`) is **THE seam**: `start_proxy` (byte-duplex +
  process launch), `build_command`, `upload_directory`, platform/version probes. `SshRemoteConnection`,
  `WslRemoteConnection`, `DockerExecConnection`, and `MockConnection` are interchangeable behind it — **a dev
  container is just another remote host** (`docker exec` the proxy, `docker cp` the binary; the `dev_container`
  crate only *provisions* then hands off to the generic Docker connection).
- **`HeadlessProject`** (`remote_server/headless_project.rs:52`) is the proof that `Project` is UI-/host-
  agnostic: **the identical store composition** (`WorktreeStore::local(fs).shared(…)`, `BufferStore`,
  `LspStore::new_local` — real language servers run here — `GitStore`, `TaskStore`, `DapStore`) minus any
  window/view, wired to an `AnyProtoClient` session. A **two-process design** (short-lived per-connection
  `proxy` ↔ a persistent `run` daemon over unix sockets) + a `ChannelClient` ack/replay buffer gives
  **reconnection** across ssh drops and multi-window session sharing.
- **`ConnectionState`** = `Connecting|Connected|Reconnecting|Disconnected|HeartbeatMissed`; 5s heartbeat, 5
  misses → reconnect (≤3 attempts).

### Marley today  ·  the gap (a difference of *kind*, not degree)

`marley_remote` deliberately does the opposite: **spawn the user's `ssh` as a normal terminal pane**
(`TerminalSession` running `ssh`), argv-safe against option-smuggling, holding **no secrets** (ssh owns all
auth). A remote pane is an opaque byte stream — **no remote filesystem, no remote file tree, no remote search,
no remote language servers.** This is a sound, tiny, secure design *for a terminal*, and it is a different thing
entirely from Zed's remote *editing*. Marley's memory even records this as intentional ("Marley's approach to
remote is deliberately thin"). The gap is not a missing feature — it is a **product-scope decision**: does
Marley want to *edit* on remote hosts, or only *shell* into them?

### Reimplementation on our stack — staged, intake-gated  `[Marley-original]`

Marley's "remote-connection seam" intake pillar is exactly Zed's `RemoteConnection` trait. Recommended posture:

- **Keep the terminal-pane ssh as-is** for the shell use case — it is strictly better (lighter, secure) when the
  user just wants a remote prompt. Do not replace it.
- **If/when remote *editing* is greenlit**, adopt the **seam, not the whole engine**: define a Marley
  `RemoteConnection` (a framed byte pipe over the *same* system-ssh shell-out Marley already does, + a small
  proto) and reuse it for SSH/Docker. This is incremental over `marley_remote`'s existing argv-safe ssh
  command-builder.
- **Defer the headless daemon.** The full `HeadlessProject` (remote fs + remote LSP + server-side search) is a
  very large surface — the server links nearly the whole editor. Marley should ship *local* worktree/search/hub
  (§1-§4) first, keep every store's `Remote{proto}` arm as a dormant seam (§3), and only build the daemon when
  an intake proves remote editing is worth it. A cheaper interim: a **remote-fs-only `Fs` impl** (proto
  `read`/`write`/`watch` to a minimal agent) would give a remote file tree + remote search *without* remote LSP
  — a natural first slice that reuses §1's `Arc<dyn Fs>` seam exactly.

### Sequencing (all deferred behind the intake)
1. (Only if remote editing is chosen) A `RemoteConnection` trait + framed pipe over Marley's existing ssh
   shell-out; a minimal proto.
2. A remote-`Fs` impl (remote file tree + remote search reuse §1/§2/§4 unchanged) — *before* any daemon.
3. A headless agent + store-`Remote` arms — only if remote LSP/tasks justify the weight.

### Provenance
`[permissive/public]` — the system `ssh`/`scp`/`sftp` binaries; `askpass`; `prost`/`prost-build` (protobuf).
`[Zed-derived]` — the headless-server + thin-client + proto-mirrored-snapshot model, the `RemoteConnection`
seam, the two-process reconnect design, `HeadlessProject`'s store reuse. `[Marley-original]` — the existing
ssh-as-terminal-pane approach (which is *not* Zed-derived and stays clean), and the staged
seam-then-remote-fs-then-daemon plan.

---

## Consolidated roadmap (dependency-ordered)

| # | Milestone | Depends on | Value delivered | Provenance weight |
|---|---|---|---|---|
| 1 | **`marley_fs`**: `trait Fs` + `RealFs` + `FakeFs` (no watch) | — | testable file layer; the substrate for everything | mostly `[public]` (notify later) + `[Zed-derived]` pattern |
| 2 | `marley_fs` **`watch()`** (notify + poll fallback) | 1 | live fs events | `[public]` notify |
| 3 | **`marley_worktree`**: `Entry` + `Snapshot` + scanner (initial) | 1 | one source of truth for project files; finder reads it | `[Zed-derived]` model |
| 4 | worktree **live watching** + `ignore` crate + `IgnoreStack` | 2,3 | tree reflects disk; real `.gitignore` | `[public]` ignore + `[Zed-derived]` |
| 5 | **project-wide search engine** (query + streaming pipeline) | 1,3 | find across files (headless) | `[public]` regex/aho + `[Zed-derived]` pipeline |
| 6 | **Phase-1 read-only results dock** (`file:line`, open-at-match) | 5 | **project-wide find ships** | `[Marley-original]` |
| 7 | in-file find upgrade (regex/whole-word/replace) | 5 | richer buffer search | `[public]` + `[Zed-derived]` options |
| 8 | **`marley_project` hub** (Local-only; `Vec<Worktree>` + buffer registry + event bus) | 3 | orchestration; enables multi-workspace + shared buffers | `[Zed-derived]` façade, `[public]` gpui |
| 9 | **`MultiBuffer`** + editable results + replace-all | 5,8, editor | edit-in-results; reused by diagnostics later | `[Zed-derived]` multibuffer *(gate on editor-as-peer intake)* |
| 10 | remote seam → remote-`Fs` → headless daemon | 1,5,8 | remote *editing* | `[Zed-derived]` *(gate on remote intake)* |

**Do first:** 1-4 (a live, watched, gitignore-aware file tree on a testable fs) — pure wins, no product-scope
questions, and they retire the biggest foundational gap. **Then** 5-6 (project-wide find, the flagship
user-visible capability). **Hub (8)** naturally when multi-workspace lands. **9 and 10 are intake-gated** — the
editable multibuffer belongs to the editor-as-peer milestone; remote editing belongs to the remote-connection
intake and should stay deferred behind the (excellent, keep-it) terminal-pane ssh.

---

## Key files (paths relative to the Zed clone in session scratch)

**fs** — `crates/fs/src/fs.rs` (`Fs` trait `:97`, `RealFs` `:399`, `FakeFs` `:1315`); `crates/fs/src/
fs_watcher.rs` (notify backend, poll fallback, `GlobalWatcher`); `crates/fs/src/fake_git_repo.rs` (fake
`GitRepository`).

**worktree** — `crates/worktree/src/worktree.rs` (`Worktree` `:95`, `Snapshot` `:176`, `LocalSnapshot` `:249`,
`Entry` `:3795`, `BackgroundScanner` `:4166`); `crates/worktree/src/ignore.rs` (`IgnoreStack`); `crates/
worktree/src/worktree_settings.rs` (`file_scan_exclusions` globs).

**project (hub + stores + search engine)** — `crates/project/src/project.rs` (`Project` `:213`,
`ProjectClientState` `:312`, `search_impl` `:4639`); `crates/project/src/worktree_store.rs` (`WorktreeStore`
`:185`); `crates/project/src/buffer_store.rs` (`BufferStore` `:34`); `crates/project/src/search.rs` (`SearchQuery`
`:65`, `SearchResult` `:22`, `SearchInputs` `:40`); `crates/project/src/project_search.rs` (`Search`,
`SearchResultsHandle`, the worker pipeline); `crates/project/src/search_history.rs`.

**search (UI + multibuffer results)** — `crates/search/src/project_search.rs` (`ProjectSearch`,
`ProjectSearchView`, `consume_search_stream`, `set_anchored_excerpts_for_path`); `crates/search/src/
buffer_search.rs` (`BufferSearchBar`); `crates/search/src/search.rs` (`SearchOptions` bitflags, actions);
`crates/multi_buffer/src/multi_buffer.rs` (`MultiBuffer` `:73`, `ExcerptRange` `:842`) + `path_key.rs`
(`PathKey`, `set_anchored_excerpts_for_path`); `crates/editor/src/items.rs` (`SearchableItem::replace_all`).

**remote** — `crates/remote/src/remote_client.rs` (`RemoteClient` `:328`, `RemoteConnection` trait `:1589`,
`ChannelClient` `:1674`, `ConnectionState` `:307`); `crates/remote/src/transport.rs` + `transport/{ssh,wsl,docker,
mock}.rs`; `crates/remote/src/protocol.rs` (length-prefixed `Envelope` framing); `crates/remote_server/src/
headless_project.rs` (`HeadlessProject` `:52`); `crates/remote_server/src/server.rs` (two-process daemon);
`crates/remote_server/src/main.rs`; `crates/dev_container/` (container provisioning → Docker connection).

**Marley baseline (this repo)** — `crates/marley_project/src/lib.rs` (`Project`, `list_files_in`, `FileTree`);
`crates/marley_search_core/src/lib.rs` (`nucleo` fuzzy); `crates/marley_app/src/command_bar.rs`
(`search_everything` — name search); `crates/marley_app/src/find.rs` (`find_matches` — in-file substring);
`crates/marley_remote/src/lib.rs` (ssh-as-terminal-pane); `crates/editor/src/buffer.rs` (single `Buffer`).
