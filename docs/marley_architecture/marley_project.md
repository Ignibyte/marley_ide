# `marley_project`

> Per-crate architecture note — **round 4 refresh · 2026-07-12 · current to M15.**
> Provenance: **`[Marley-original]`** (INVENT, std-only — no Warp lineage). The M2 project-discovery model
> that file listing, fuzzy file-open ([`marley_search_core`](./marley_search_core.md)), and the workspace
> docks hang off of. Pure and gpui-free.

The project model — the git repository (or plain directory) the terminal is working in. The M2 "moat"
foundation: the discovered `Project` is the context for file listing, search, and the workspace docks.

## Surface

```rust
pub struct Project { pub root: PathBuf, pub name: String, pub is_git: bool }
impl Project { pub fn discover_in(start: &Path) -> Project }
pub fn resolve_under_root(root: &Path, path: &Path) -> PathBuf;   // M12.1 #190 — CWD-independent file reads
pub fn canonical_under_root(root: &Path, path: &Path) -> PathBuf; // M20 #319 — THE stored editor-file identity
pub fn canonical_root(root: &Path) -> PathBuf;                    // M20 #319 — the root-spelling half (fails OPEN)
pub fn rel_under_root<'a>(root: &Path, path: &'a Path) -> &'a Path; // M20 #319 — the inverse (pathspec/reveal/labels)
```

- **`discover_in(start)`** — walk UP from `start` (then each `.parent()`), returning the **nearest**
  ancestor that holds a `.git` entry as `root`, with `is_git = true`. `.git` is detected with `.exists()`,
  so a `.git` **file** (git worktrees + submodules point `.git` at a file) counts, not just a `.git`
  directory. When no `.git` is found up to the filesystem root, `root = start` and `is_git = false` —
  Marley still has a usable project in a plain directory. `name` is the root's basename (or its whole path
  for a root like `/`, via the private `name_for`).
- **`resolve_under_root(root, path)`** (M12.1 #190) — join a RELATIVE `path` (as `FileTree::path_at` /
  `list_files_in` produce) onto `root` to get a **CWD-independent absolute path**; an already-absolute
  `path` (e.g. a terminal file-reference) is returned unchanged. **Why it exists:** a bundled app is
  launched by LaunchServices with CWD `/`, so reading a root-relative path with `std::fs` resolved it
  against `/`, the read silently failed, and a file click appeared to do nothing. This composes safely with
  every caller (relative → re-rooted; absolute → passthrough).
- **`canonical_under_root(root, path)`** (M20 #319) — `resolve_under_root`, then canonicalize when the
  target exists; the resolved join when it does not. **THE stored identity of an open editor file**: it
  collapses root-symlink aliases (macOS `/tmp`→`/private/tmp`, a symlinked `~/dev`), so a file reached via
  the tree (root-relative) and via an LSP jump (server-canonical uri) stores ONE `PathBuf` — one buffer,
  and raw `==` over stored paths is sound. The missing-file fallback is deliberately the SAME one the
  app's `same_file` compare uses, so storage and compare can never disagree. `LspHost::absolute` delegates
  here — one normalization algorithm in the workspace.
- **`canonical_root(root)`** (M20 #319) — the root-spelling half, shared by every seam comparing a STORED
  (canonical) path against a verbatim-kept root (host-spawn containment, the ⌘⇧F search walk, rel
  displays). Fails **OPEN** (verbatim fallback on a vanished root) — fine for identity/display; a
  write-containment guard keeps its own fail-closed shape.
- **`rel_under_root(root, path)`** (M20 #319) — the inverse: a stored path back to root-RELATIVE for git
  pathspecs, tree reveal, and display labels. Strips the verbatim root, else the canonical root, else
  returns the path unchanged (outside the project); every arm borrows from `path`.

## Design notes
- **Nearest, not top-most** — the innermost repo wins, matching `git rev-parse --show-toplevel` for the
  common case. A monorepo/sub-repo policy beyond "nearest `.git`" is deferred.
- **`name_for` is a separate fn** so its `file_name() == None` (`/`) fallback branch is directly
  unit-testable — inside `discover_in` that branch is uncoverable.
- **Mutation caveat** — cargo-mutants does not mutate `.exists()`/`.parent()`/struct-field assignments, so
  `discover_in`'s walk yields no viable mutants; its correctness is guarded by behavioral tests over
  `tempfile` trees (nested `.git`, a `.git` file, a no-repo tree) + 100% coverage. `name_for` +
  `resolve_under_root` carry the crate's viable mutants (the latter's branch-swap / root-ignoring mutants
  are killed by explicit relative-vs-absolute + different-root assertions); the #319 identity helpers'
  mutants (fallback-arm swaps, strip-arm deletes) are killed by explicit symlink-fixture tests
  (alias-collapse, missing-file join fallback, verbatim/canonical/outside strips).

## File listing (M2.A #55)

```rust
pub fn should_skip(component: &str) -> bool;                 // .git | node_modules | target | .DS_Store
pub struct FileListing { pub files: Vec<PathBuf>, pub truncated: bool }
pub const MAX_FILES: usize = 10_000;
pub fn list_files_in(root: &Path) -> FileListing;           // = list_files_capped(root, MAX_FILES)
```

- **`list_files_in`** — a stack walk from `root`: every non-skipped regular file as a path relative to
  `root`, sorted. Skipped directories (`should_skip`) are pruned (subtree not walked). The dir-vs-file
  decision uses `DirEntry::file_type()`, which does NOT follow symlinks — so a symlink to an ancestor is
  ignored rather than recursed, and the walk can never loop. Unreadable directories are skipped (no panic).
  Bounded at `MAX_FILES`; the `truncated` flag is the honest "first N of many" signal (no silent cap).
  Which N survives truncation is traversal-order-dependent (unspecified) — fine for the first cut. A curated
  `should_skip` set is the first cut — a full `.gitignore` parser is deferred.
- The private `list_files_capped(root, cap)` exposes the bound to tests (truncation is verified with a
  small cap, not a 10k-file tree).

## File tree (M2.A #56)

```rust
pub struct TreeRow { pub depth: usize, pub name: String, pub is_dir: bool, pub collapsed: bool }
pub struct FileTree { /* private Vec<Node>; Default */ }
impl FileTree {
    pub fn from_files(files: &[PathBuf]) -> FileTree;   // nest + merge; dirs-before-files, alpha
    pub fn visible_rows(&self) -> Vec<TreeRow>;         // collapse-aware pre-order flattening
    pub fn toggle(&mut self, index: usize);            // flip the dir at a visible-row ordinal
    pub fn path_at(&self, index: usize) -> Option<PathBuf>; // the row's full relative path (#59)
}
```

- **`from_files`** nests the flat listing by path component (non-terminal = dir, last = file), merging
  shared prefixes; each level is ordered dirs-before-files then alphabetical; dirs start expanded.
- **`visible_rows`** is what the dock render walks: one row per node, depth-tagged, a collapsed dir's
  descendants omitted. **`toggle(index)`** and **`path_at(index)`** walk the SAME pre-order, so a rendered
  row's ordinal maps straight to its node (toggle flips that directory's `collapsed`; `path_at` joins the
  ancestor names down to the row).
- **App wiring:** `RootView` builds a `FileTree` once at startup (`Project::discover_in(cwd)` →
  `list_files_in` → `from_files`); the Left "Files" dock renders `visible_rows` with a `▾`/`▸` disclosure,
  a click on a dir row calls `toggle`, and a click on a file row opens it via `path_at` →
  `resolve_under_root` (the #190 fix above). The render + click are the coverage-excluded,
  self-test-verified shim; the model is cov/MSI 100. NB: a directory-aware toggle needs a
  mid-collapse-then-toggle test or the recurse-guard mutants (`&&`/`!`/cursor `+=`) survive.

## Consumers
Search lives separately in [`marley_search_core`](./marley_search_core.md) (#54); fuzzy file-open (#57)
ranks these listings; the Right "Details" dock (#58) is the dock counterpart. Note this is fuzzy **name**
matching over the listing — content search is a named gap (see the search-core note).
