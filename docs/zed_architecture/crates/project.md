# project

> Per-crate reference (Marley round 2) — crate dir `crates/project` (package `project`, lib root
> `src/project.rs`). Zed is the EDITOR reference for Marley's editing surface. Source cloned into session scratch
> only; this is Marley's own description, never Zed code.

| | |
|---|---|
| **Subsystem** | [06 — Project, FS, Worktree & Search](../subsystems/06-project-fs-search.md) |
| **License** | **GPL-3.0-or-later** (per-crate `LICENSE-GPL`) |
| **Provenance** | `[Zed-derived]` façade+router + host-agnostic store composition; the **search engine** composes `[permissive/public]` `aho-corasick`/`fancy-regex`/`globset`/`wax` |
| **Key external deps** | `aho-corasick` (MIT/Unlicense), `fancy-regex` (MIT), `regex` (MIT/Apache), `globset` + `wax` (MIT), `fuzzy_nucleo`, `sha2`, `toml`, `which` |
| **Zed-internal deps** | `fs`, `worktree`, `lsp`, `dap`, `git`, `git_hosting_providers`, `context_server`, `remote`, `rpc`/`client`, `prettier`, `node_runtime`, `task`, `terminal`, `snippet`, `extension`, `buffer_diff`, `sum_tree`, `settings`, `gpui` |
| **Zed dependents** | ~71 crates (the most-depended-on non-leaf crate; the workspace/editor spine) |
| **Marley target** | **`marley_project` hub** (grow incrementally, Local-only) + a headless **search engine** |

## Purpose

`crates/project` is **THE hub** — a gpui `Entity` that **owns almost no domain data** and instead acts as a
**thin façade + event router** over ~15 sub-store entities, wiring them together and translating their narrow
events into one workspace-facing stream. It is also **host-agnostic by construction**: the *same* code drives a
local disk, an SSH daemon, or a collab host by swapping a type-erased RPC client. Two things live here that
matter most to Marley: **(a)** the orchestration/host-abstraction layer, and **(b)** the headless **project
search engine** (query model + fan-out pipeline) — the flagship capability Marley entirely lacks.

**Naming caution:** Marley's current `Project{root, name, is_git}` corresponds to Zed's **`Worktree`**, *not* to
this crate. Zed's `Project` = a hub owning *many* worktrees; Marley has **no counterpart** today.

## Key types, modules & public API

**`src/project.rs`** (6.8k lines) — the hub.

- **`struct Project`** (`:213`) — a gpui `Entity` holding handles, not data: `worktree_store`, `buffer_store`,
  `lsp_store`, `git_store`, `dap_store`, `task_store`, `image_store`, `toolchain_store`, `context_server_store`,
  `agent_server_store`, `bookmark_store`, `breakpoint_store`, `terminals`, `environment`, `settings_observer`,
  `snippets`, plus `fs: Arc<dyn Fs>`, `collab_client`, `remote_client: Option<Entity<RemoteClient>>`,
  `client_state: ProjectClientState`, and three `SearchHistory` (query/include/exclude). Most public methods are
  one-line delegations (`open_buffer` → `buffer_store.update(…)`, `:3196`; `open_path`, `:3084`; `worktrees`,
  `:2381`; `search`, `:4675`). The value is in the two cross-cutting layers below.
- **Host-agnostic by construction (the central idea).** `enum ProjectClientState{ Local, Shared{remote_id},
  Collab{sharing_has_stopped, capability, remote_id, replica_id} }` (`:312`) × an
  `Option<Entity<RemoteClient>>`. Predicates: `is_local()` (`:2998`), `is_via_remote_server()` (`:3009`, ssh),
  `is_via_collab()` (`:3020`). Constructors `Project::local(…)` (`:1174`) / `Project::remote(ssh)` (`:1379`) /
  `in_room(collab)` differ **only** in which client each store is handed. Every *store* carries its own
  `enum State{ Local{fs} | Remote{upstream_client: AnyProtoClient, project_id, path_style} }` — because
  `AnyProtoClient` is a type-erased protobuf RPC client with a uniform request/send/subscribe surface, the store
  *logic is identical* whether the peer is absent (local fs), an SSH daemon, or a collab host. **You swap the
  client, not the code.**
- **Event translation + proto mirroring.** Each store `impl EventEmitter<StoreEvent>`; the `Project`
  `cx.subscribe`s to all of them and fans their events into **one `enum project::Event`** (`:334`, ~40 variants:
  `WorktreeAdded`/`WorktreeUpdatedEntries`/`WorktreeRemoved`, `DiagnosticsUpdated`, `LanguageServerAdded`,
  `ActiveEntryChanged`, `CollaboratorJoined`, `Toast`, …) — the single stream a `Workspace` subscribes to —
  while simultaneously forwarding mutations onto the proto wire so a host can serve guests.
- **`struct ProjectPath{ worktree_id: WorktreeId, path: Arc<RelPath> }`** (`:426`) — the canonical (worktree, rel
  path) address used everywhere a file is referenced across worktrees.

**The stores (each a peer `Entity`, each `Local|Remote`):**
- **`src/worktree_store.rs`** — `struct WorktreeStore` (`:185`) holds `worktrees: Vec<WorktreeHandle>`,
  `loading_worktrees` (coalesces concurrent opens via `Shared<Task>`), and `state: enum WorktreeStoreState{
  Local{fs} | Remote{upstream_client, upstream_project_id, path_style} }` (`:153`). Dispatches
  `create_worktree` local-vs-remote; emits `enum WorktreeStoreEvent{ WorktreeAdded, WorktreeUpdatedEntries,
  WorktreeUpdatedGitRepositories, WorktreeRemoved, … }` (`:200`).
- **`src/buffer_store.rs`** — `struct BufferStore` (`:34`) keys open buffers by `BufferId` with **two indexes**:
  `opened_buffers: HashMap<BufferId, OpenBuffer>` + `path_to_buffer_id: HashMap<ProjectPath, BufferId>`, and
  (Local arm) `local_buffer_ids_by_entry_id: HashMap<ProjectEntryId, BufferId>` — the tie from a buffer back to
  a worktree `Entry`. `state: enum BufferStoreState{ Local(LocalBufferStore) | Remote(RemoteBufferStore) }`
  (`:63`). Concurrent opens coalesced via `loading_buffers` (`Shared<Task>`). Emits `BufferAdded`/`BufferDropped`/
  `BufferChangedFilePath`. `struct ProjectTransaction(HashMap<Entity<Buffer>, Transaction>)` is the multi-buffer
  atomic-edit unit (used by rename, replace-all, workspace-edit).
- **`src/lsp_store.rs`** (15k lines — the largest) — `enum LspStoreMode{ Local | Remote | … }` with
  `LocalLspStore` (`:296`, real language servers) vs `RemoteLspStore` (`:4085`); `new_local`/`new_remote`
  (`:4377`) prove the same façade drives real servers locally or proxied remotely. (Out of Marley scope
  initially — noted for the pattern.)
- Other stores: `git_store.rs` (10k — status/blame/stage, the owner of git status the worktree deliberately
  omits), `dap_store`, `task_store`/`task_inventory`, `image_store`, `toolchain_store`, `context_server_store`,
  `agent_server_store`, `environment`, `terminals`, `bookmark_store`, `trusted_worktrees`, `manifest_tree`.

**The headless search engine** (UI-free — the `search` crate renders it):
- **`src/search.rs`** (758 lines) — the **query model.** `enum SearchQuery` (`:65`) with two variants sharing an
  `inner: SearchInputs`:
  - **`Text{ search: AhoCorasick, replacement, whole_word, case_sensitive, include_ignored, inner }`** — literal
    search via **`aho-corasick`** (streams over bytes, no String alloc). `SearchQuery::text(…)` (`:98`) **falls
    back to regex** for case-insensitive non-ASCII (aho-corasick can't Unicode-case-fold).
  - **`Regex{ regex: fancy_regex::Regex, multiline, whole_word, case_sensitive, include_ignored,
    one_match_per_line, escaped, inner }`** — via **`fancy-regex`** (backtracking; lookaround/backrefs).
  - **`struct SearchInputs{ query: Arc<str>, files_to_include: PathMatcher, files_to_exclude: PathMatcher,
    match_full_paths, buffers: Option<Vec<Entity<Buffer>>> }`** (`:40`) — include/exclude are compiled globs
    (**`wax`/`globset`**); `buffers: Some(_)` scopes to "open buffers only."
  - Match ops: `search_str(text) -> Vec<Range<usize>>` (`:707`), `replacement_for(text) -> Option<Cow<str>>`
    (`:475`, regex capture substitution vs literal). **`enum SearchResult`** (`:22`) is a *stream* with control
    signals: `Buffer{ buffer: Entity<Buffer>, ranges: Vec<Range<Anchor>> }` + `Searching` / `WaitingForScan` /
    `LimitReached`. **Matches are `Anchor` ranges bound to a live `Buffer`, not raw file offsets** — this is
    *why* results are inherently editable in the UI crate.
- **`src/project_search.rs`** (1.1k lines) — the **execution pipeline.** `Project::search` returns a `#[must_use]`
  `struct SearchResultsHandle` (`:62`); `Search::into_handle` (`:158`) picks local | remote | open-buffers-only.
  **It is NOT ripgrep** — a hand-rolled multi-stage async pipeline of `async_channel`s fanned across the gpui
  `BackgroundExecutor` with `(num_cpus-1).max(1)` `Worker`s (`:351`, `:659`) in a `select_biased!` loop that
  drains expensive full-scans before cheap probes. Stages: `provide_search_paths` (`:418`, per worktree await
  scan-complete, stream every candidate `Entry`) → `handle_scan_path` (`:811`, cheap glob filter, no I/O; **if
  the entry is an open buffer, skip disk and search the in-memory version** — dirty buffers never read stale
  disk) → `handle_find_first_match` (`:773`, `fs.open_sync` + first-match-only, **rejects invalid-UTF-8/binary
  early**) → `handle_find_all_matches` (`:746`, full match over the buffer snapshot → `Vec<Range<Anchor>>`).
  Caps: `MAX_SEARCH_RESULT_FILES = 5_000`, `MAX_SEARCH_RESULT_RANGES = 10_000` (`:154`) → `LimitReached`.
- **`src/search_history.rs`** (128 lines) — `struct SearchHistory` + `SearchHistoryCursor` (per-input
  `next`/`previous`/`current` + a draft slot); `enum QueryInsertionBehavior`. Three instances on `Project`
  (query/include/exclude).

## Depends on (internal)

- [`worktree`](./worktree.md) — the file model each `WorktreeStore` holds; search fans over `Snapshot` entries.
- [`fs`](./fs.md) — `Arc<dyn Fs>` for the `Local` arm of every store; `open_sync` in the search probe stage.
- `buffer_store`↔`language::Buffer` — the open-buffer registry; `lsp`/`dap`/`git`/`task`/`terminal` — the other
  stores. `rpc`/`client`/`remote` — the `AnyProtoClient` seam + ssh backend. `gpui` — `Entity`/`EventEmitter`/
  `BackgroundExecutor` (supplied directly; no reimplementation needed).

## Used by (internal)

~71 crates — `workspace`, `editor`, `search`, the panels, the agent, and the app all route through the hub. It
is the backend every window talks to.

## ★ How this maps to Marley's Workspace → Project → Tab

The pivotal architectural question — the mapping is **not 1:1**:

| Zed | Role | Marley equivalent |
|---|---|---|
| **`Workspace`** (separate crate) | the **window**: `PaneGroup` + panes + docks, holds **one** `Entity<Project>` | Marley's `RootView` / the Workspace→Project→Tab shell |
| **`Project`** (this crate) | the **backend hub**: owns **many** worktrees + buffers + LSP + search + host-abstraction | **has no equivalent** — Marley has no hub |
| **`Worktree`** | **one** opened root folder + its live tree | **Marley's `Project{root, name, is_git}`** |

So **Marley's "Project" is really Zed's "Worktree,"** and Marley's orchestration lives ad hoc in `RootView`.
Zed's `Project` layer — a hub owning *N* worktrees behind a host-agnostic seam — has **no counterpart**.

**Does a terminal-first tool need the hub? Not yet — but it is exactly where chad's workspace-centric direction
lands.** Three forces pull a lightweight hub into existence, two already on Marley's roadmap:
1. **Multi-workspace / launcher (M13 "Workspace Cockpit" — multiple workspaces open like PhpStorm; the rail
   highlights the active one).** That is *precisely* Zed's `Workspace(window) → Project(hub) → Worktree(folder)`
   shape. A per-workspace holder owning its worktree(s) + open buffers is the natural home for "which folders are
   open here," and the rail highlight is a `ProjectEvent` subscription.
2. **Project-wide search** needs *someone* to own "all worktrees + all open buffers" to fan a query across.
3. **Editor-as-peer (M14 intake)** means buffers are shared between tree, finder, search results, and terminal
   cwd — a `BufferStore`-like registry (keyed by `ProjectPath`, tied to `Entry`) prevents three subsystems
   opening three copies of one file.

## Marley today · gap

`Project` is a value struct (`{root, name, is_git}` + file-listing helpers), not an entity, not a hub. There is
no open-buffer registry (the editor holds one `Buffer`), no event bus unifying tree/finder/terminal, no seam for
multiple roots or a remote backend, and **zero content search**. Gap = the entire orchestration +
host-abstraction layer, plus the search engine.

## Reimplementation on our stack  `[Marley-original]`

**Grow a minimal `marley_project` hub incrementally — implement only the `Local` arm** of the per-store `State`
enum, and *keep the `Remote{proto}` seam as an unimplemented hook* so remote can slot in later without touching
call sites. Do **not** import collab/LSP/DAP.

- **The hub:** promote `marley_project` to a gpui `Entity` owning `worktrees: Vec<Worktree>` + an
  `open_buffers: HashMap<ProjectPath, BufferHandle>` registry (two-index like `BufferStore`: also key by
  `EntryId`), emitting a single `ProjectEvent` the `RootView`/rail subscribes to. Local-only.
- **The search engine (separately, sooner):** a `SearchQuery{ Text(AhoCorasick) | Regex(fancy_regex) }` +
  `SearchInputs{ include/exclude via globset }` + a streaming `SearchResult`, reusing the pipeline shape over
  Marley's `background_executor` (candidate → glob filter → cheap first-match probe over `marley_fs` with
  binary-skip → full match). **Keep the open/dirty-buffer precedence** even single-user (the editor may have
  unsaved edits). Cap results. This is `[permissive/public]` algorithms composed by a `[Zed-derived]` pipeline —
  pure, `FakeFs`-driven, cov/MSI-100-friendly. Results anchored to buffers, not offsets (the UI crate decides
  read-only vs editable — see [`search.md`](./search.md)).

### Sequencing
1. Headless **search engine** (query + streaming pipeline over worktree + fs + open-buffers). Testable, ships value with a read-only dock.
2. `marley_project` **hub** (Local: `Vec<Worktree>` + buffer registry + `ProjectEvent`) — when multi-workspace lands.
3. Route the finder + project-search through the hub's worktrees; one hub per workspace, launcher/rail iterate hubs.
4. Keep the `Remote{proto}` arm dormant for a future remote seam.

## Provenance

- `[permissive/public]` — **`aho-corasick`**, **`fancy-regex`**, **`regex`**, **`wax`/`globset`**, `nucleo`
  (already vendored in Marley); **`gpui`** (Apache-2.0; Marley already depends on it) supplies
  `Entity`/`Context`/`EventEmitter`/`BackgroundExecutor` directly.
- `[Zed-derived]` (patterns, clean-room) — the façade+router hub; the host-agnostic per-store
  `State{Local|Remote}` + `AnyProtoClient` seam; the two-index buffer↔entry registry; the
  `Workspace(window)→Project(hub)→Worktree(folder)` layering; the 2-variant `SearchQuery` enum + the
  candidate→probe→match streaming pipeline with **open/dirty-buffer precedence** and binary-skip; the caps.
- `[Marley-original]` — the scoped Local-only hub and the down-scoped engine.

## Notes / gotchas

- **The hub owns *handles*, not data** — its value is subscription-fan-out + host-abstraction, not storage.
  Don't fold worktree/buffer data into it.
- **`AnyProtoClient` is the whole trick** — one type-erased RPC client makes local/ssh/collab the *same* store
  code. Even if Marley only ships Local, model the store as `State{Local|Remote}` so the seam exists.
- **Search results are `Anchor` ranges on a live `Buffer`** — that single decision is what later makes editable
  results free. Bind to buffers, not file offsets, from day one.
- **Open-buffer precedence in search** — always search the in-memory buffer if one is open, never stale disk.
  Matters even for a single user with unsaved edits.
- **`#[must_use] SearchResultsHandle`** — the caller must declare interest (matching-buffers vs full matches);
  keep that two-phase shape so remote delegation (server streams ids, client runs full pass) stays possible.
