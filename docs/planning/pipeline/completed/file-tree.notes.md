# Files dock — real project file tree — Notes

- **Forge ticket:** #56 `1140cdd5-1102-4eb8-a109-c3aad7233c4f`
- **AAR:** `c8e2300f-ce2c-4ae1-95be-244b8947cdfd`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-056-file-tree.md

## Phase 1 — Plan
- **Request:** forge #56 (M2.A seq-4, auto-approved) — the Files dock file tree. FIRST M2.A UI ticket.
- **Classification:** work pipeline, `feature`, PURE `FileTree` (marley_project) + an app.rs SHIM. UI —
  validate MUST self-test-capture.
- **Discovery (§18):** `RootView` (app.rs:53) holds `docks: [DockState; 2]`; built in `new()` (141);
  `render()` (727) calls `dock_panel(side, x, w, h, colors)` (93) per open dock — the content area is an
  empty `div().flex_1().p_3()` (line 122, comment already says "M2 fills it with a file tree/details").
  So the tree renders inside dock_panel for the Left dock.
- **Wiring:** `new()` already spawns the session from `std::env::current_dir()`; reuse that cwd for
  `Project::discover_in` → `list_files_in(project.root)` → `FileTree::from_files`. Store on RootView.
- **Toggle coordinate (D2):** the flat `visible_index` — the render enumerates rows, so a click knows
  its ordinal; toggling by visible index keeps in sync with collapsed-hiding.
- **Hollow-MSI watch (#53):** `from_files`/`visible_rows`/`toggle` have real branching (sort, collapse
  test, index match) → viable mutants; the recursion is fine.
- **AAR id:** `c8e2300f-ce2c-4ae1-95be-244b8947cdfd`.

## Phase 2 — Design

### PURE — ADD to `crates/marley_project/src/lib.rs`
```rust
/// One rendered row of a [`FileTree`] — a flattened, collapse-aware view for the dock render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow { pub depth: usize, pub name: String, pub is_dir: bool, pub collapsed: bool }

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node { name: String, is_dir: bool, collapsed: bool, children: Vec<Node> }

/// A collapsible tree of a project's files, built from the flat [`list_files_in`] output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileTree { roots: Vec<Node> }

impl FileTree {
    /// Nest `files` by path component (non-terminal components are directories, the last is a file),
    /// merging shared prefixes; each level is ordered dirs-before-files then alphabetical. Dirs start
    /// expanded.
    pub fn from_files(files: &[PathBuf]) -> FileTree {
        let mut roots: Vec<Node> = Vec::new();
        for path in files {
            let parts: Vec<String> = path.iter().map(|c| c.to_string_lossy().into_owned()).collect();
            if !parts.is_empty() {
                insert(&mut roots, &parts);
            }
        }
        sort_level(&mut roots);
        FileTree { roots }
    }

    /// A pre-order, collapse-aware flattening: one row per node (depth from 0); a collapsed dir's
    /// descendants are omitted.
    pub fn visible_rows(&self) -> Vec<TreeRow> {
        let mut out = Vec::new();
        visit(&self.roots, 0, &mut out);
        out
    }

    /// Flip the `collapsed` flag of the DIRECTORY at visible-row `index`; a no-op for a file row or an
    /// out-of-range index.
    pub fn toggle(&mut self, index: usize) {
        let mut cursor = 0;
        toggle_at(&mut self.roots, index, &mut cursor);
    }
}

fn insert(nodes: &mut Vec<Node>, parts: &[String]) {
    let (head, rest) = parts.split_first().expect("non-empty");
    let is_dir = !rest.is_empty();
    let idx = match nodes.iter().position(|n| n.name == *head) {
        Some(i) => i,
        None => {
            nodes.push(Node { name: head.clone(), is_dir, collapsed: false, children: Vec::new() });
            nodes.len() - 1
        }
    };
    if is_dir {
        insert(&mut nodes[idx].children, rest);
    }
}

fn sort_level(nodes: &mut [Node]) {
    nodes.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    for n in nodes.iter_mut() {
        sort_level(&mut n.children);
    }
}

fn visit(nodes: &[Node], depth: usize, out: &mut Vec<TreeRow>) {
    for n in nodes {
        out.push(TreeRow { depth, name: n.name.clone(), is_dir: n.is_dir, collapsed: n.collapsed });
        if n.is_dir && !n.collapsed {
            visit(&n.children, depth + 1, out);
        }
    }
}

fn toggle_at(nodes: &mut [Node], target: usize, cursor: &mut usize) -> bool {
    for n in nodes.iter_mut() {
        if *cursor == target {
            if n.is_dir {
                n.collapsed = !n.collapsed;
            }
            return true;
        }
        *cursor += 1;
        if n.is_dir && !n.collapsed && toggle_at(&mut n.children, target, cursor) {
            return true;
        }
    }
    false
}
```
`insert`'s `split_first().expect(...)` is safe (caller guarantees non-empty) — NOT an input-reachable
unwrap (§14 ok). `toggle_at` walks the SAME order as `visit`, so `index` is a visible-row ordinal.

### SHIM — `crates/marley_app/src/app.rs` (mutants::skip + coverage-excluded)
- `RootView` gains `file_tree: FileTree`; `new()` builds it: `let cwd = env::current_dir().unwrap_or…;
  let project = Project::discover_in(&cwd); let listing = list_files_in(&project.root); file_tree =
  FileTree::from_files(&listing.files);`.
- `dock_panel` gains a `content: gpui::Div` param for its content area (was the empty `div().flex_1()`).
- `render()` builds the Left dock content from `self.file_tree.visible_rows().iter().enumerate()`: each
  row a `div` padded-left `row.depth * INDENT`, a disclosure glyph (▾ expanded / ▸ collapsed) for dirs
  (blank for files), then `row.name`; a dir row gets `.on_mouse_down(Left, cx.listener(move |this, _,
  _, cx| { this.file_tree.toggle(i); cx.notify(); }))`. Right dock content = empty div (until #58).
- `marley_app/Cargo.toml` += `marley_project = { path = "../marley_project" }`.

### File manifest
- MODIFY `crates/marley_project/src/lib.rs` — FileTree/TreeRow/Node + from_files/visible_rows/toggle + tests.
- MODIFY `crates/marley_app/src/app.rs` — RootView.file_tree, new() wiring, dock_panel content param, Left-dock render + click.
- MODIFY `crates/marley_app/Cargo.toml` — add marley_project dep.

### Mutation Targets (pure)
- `insert` — `!rest.is_empty()` (dir-vs-file), the position find-or-create (merge). `sort_level` —
  `b.is_dir.cmp(&a.is_dir)` (dirs first) + `.then name` (alpha). `visit` — `depth+1`, the
  `is_dir && !collapsed` recurse guard. `toggle_at` — `*cursor == target`, the `if n.is_dir` flip
  guard, `*cursor += 1`, the collapsed-recurse guard, the early return.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `from_files_merges_shared_prefix` — `["a/b/x","a/c"]` → visible_rows `[(0,a,dir),(1,b,dir),(2,x,file),(1,c,file)]` (ONE `a`, nested; b before c: dir-first) | unit |
| REQ-002 | `from_files_sorts_dirs_before_files_alpha` — `["z","a_d/f","m","b_d/g"]` → top rows order `a_d, b_d, m, z` (dirs alpha, then files alpha) | unit |
| REQ-003 | `visible_rows_hides_collapsed_children` — collapse a dir (toggle its row) → its children absent; expand → back | unit |
| REQ-004 | `toggle_dir_only_and_bounds` — toggle a dir row flips collapsed; toggle a file row = no-op; toggle out-of-range = no-op (no panic) | unit |
| REQ-005 | Left dock renders the tree; click a dir collapses | **self-test harness** (bundle+capture; drive a click) |
| REQ-006 | gate GREEN, cov/MSI 100 pure; shim excluded | gate |

Uncoverable: the app shim (`dock_panel` render, the click listener, `new()` wiring) — masked
(mutants::skip) + coverage-excluded, proven by REQ-005 self-test. Pure FileTree fully unit-covered.

### Risks / decisions
- D-2.1 `toggle` by visible index — `toggle_at` mirrors `visit`'s traversal EXACTLY (node, then visible
  children); any divergence would mismap clicks — a REQ-003+004 combined test (collapse a mid-tree dir,
  then toggle a LATER row) guards it. D-2.2 `insert` uses `expect` on a caller-guaranteed non-empty
  slice — not input-reachable; alternatively `if let Some((head,rest))` — will use `let ... else return`
  to avoid any expect if the critic prefers. D-2.3 dock_panel content param keeps click listeners in
  render() (has cx); dock_panel stays a styling shim.

## Phase 3 — Implement
- **Built (PURE, marley_project/src/lib.rs):** `TreeRow`, `Node`, `FileTree` (Default) +
  `from_files`/`visible_rows`/`toggle` + private `insert`/`sort_level`/`visit`/`toggle_at`. Used
  `let Some((head,rest)) = split_first() else { return }` (no expect, per the design lock-in).
- **Built (SHIM, app.rs — mutants::skip/cov-excluded):** import `marley_project::{list_files_in,
  FileTree, Project}`; `RootView.file_tree` field; `new()` builds it (discover_in(cwd) → list_files_in →
  from_files); `dock_panel` gained a `content: gpui::Div` param (renders it instead of the empty area);
  `render()` builds the Left dock's tree (indent by depth, ▾/▸ disclosure, `foreground` for dirs /
  `muted` for files, click a dir → `cx.listener` toggle+notify); Right dock content = empty div.
  `marley_app/Cargo.toml` += `marley_project`.
- **Deviations:** text color is `colors.foreground` (ThemeColors has no `text` field — the design said
  `colors.text`).
- **Verification:** `cargo fmt`; `cargo check -p marley_project` + `-p marley` 0 err; `cargo clippy -p
  marley -- -D warnings` exit 0. Tests + self-test are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (byte-faithful probe + the exact mid-collapse-then-toggle parity sequence + real
  cargo-mutants + coverage measurement). Verdict: **pure logic CORRECT** — no runtime bug; MSI 100
  achievable (25/25 viable). Toggle/visit parity HOLDS.
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | MED (blocks gate:4) | `insert`'s `let Some((head,rest)) = split_first() else { return }` line is UNREACHABLE via `from_files` (the `if !parts.is_empty()` guard + recursion invariant) → 99.35% line cov → gate fails. | REAL | **P4 option B:** DROP the redundant `if !parts.is_empty()` guard in `from_files` so an empty path flows into `insert`'s `else` (the single guard), coverable black-box via `from_files(&[PathBuf::from("")])`. Fold into validate (crate edits gated to that phase). |
  | F2 | LOW | cargo-mutants makes NO comparator mutant for `sort_level`; only the whole-body `sort_level→()`. REQ-002 kills it ONLY if its input order ≠ sorted order (else equivalent survivor — the #54 trap). | Accept (test discipline) | Keep REQ-002 fixture `[z, a_d/f, m, b_d/g]` UNSORTED, assert exact `[a_d,b_d,m,z]`. Already planned. |
  | F3 | LOW | `["a","a/b"]` — `a` seen as file then dir prefix → child `b` silently hidden. | Accept | UNREACHABLE from `list_files_in` (a name can't be both file+dir); degrades gracefully (no panic). Not a shipped bug. |
- **Verified (probe, RUN):** merge (one `a` node); dirs-before-files alpha sort; **toggle/visit parity
  EXACT** (`toggle(1)` after collapsing `a` hits `d` not hidden `b`; re-expand restores b/x); toggle
  file-row/OOB/empty = no-op no-panic; `nodes[index]` always in-bounds. Shim properly masked
  (dock_panel mutants::skip; app.rs cov-excluded via the gate regex); the click uses the same
  `visible_rows` ordinal as the render → consistent. §20 trivial. MSI 25/25 with the planned tests.
- **No code change now** — F1 folds into P4 (remove guard + empty-path test); F2/F3 test-discipline/accepted.

## Phase 4 — Validate
- **F1 applied:** dropped the redundant `if !parts.is_empty()` guard in `from_files` (insert's `else`
  is now the single, reachable guard).
- **Tests added** (`lib.rs`): `from_files_merges_shared_prefix` (REQ-001), `from_files_sorts_dirs_before_files_alpha`
  (REQ-002, unsorted fixture), `visible_rows_hides_collapsed_children` (REQ-003), `toggle_dir_only_and_bounds`
  (REQ-004), `from_files_empty_path_yields_empty_tree` (F1), + `toggle_recurses_and_respects_collapse`
  (the survivor-killer — see below).
- **MSI survivor fix:** the first gate run FAILED (MSI 82.6%, 4 missed) — all in `toggle_at`
  (`*cursor += 1`→`*= 1`; the recurse-guard `&&`→`||` ×2; drop `!`). My initial toggle tests only hit
  index 0 / file rows, never forcing cursor advancement + collapse-aware recursion. Added
  `toggle_recurses_and_respects_collapse` (the critic's exact mid-collapse-then-toggle sequence: toggle
  a DEEP row @ idx 4; collapse `a`, then toggle(1) must hit `d` not hidden `b`) → **MSI 100 (23/0)**.
- **Runs (actual):** `cargo nextest -p marley_project` all pass; workspace 158 passed.
- **SELF-TEST (UI — REQ-005, drove the LIVE app):** bundled + launched the bare binary FROM the project
  dir (a `.app`/LaunchServices launch gets cwd `/`, so `discover_in` found only `/` — a bare launch from
  the repo inherits the right cwd). Captured the Left "Files" dock rendering the **project tree** —
  `▾ .cargo/audit.toml`, `▾ .claude → commands/pipeline/*.md, hooks/*.sh`, `▾ crates`… indented, `▾`
  disclosure on dirs, dirs bright / files muted (`scratchpad/filetree3.png`). Then drove a CLICK on
  `.claude` (`drive.swift clickat:0.05,0.164`) → it **collapsed** (`▾`→`▸`, children hidden;
  `scratchpad/filetree_collapsed.png`). Both render + interaction verified.
- **Gotcha found + noted:** `bundle-app.sh` only builds if the binary is ABSENT → it served a STALE
  pre-#56 binary (empty dock) until I `cargo build -p marley` explicitly. Added a `clickat:` action to
  `drive.swift` for absolute-coordinate clicks (reusable for future UI tests).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0% (23/0). app.rs
  shim coverage-excluded + mutants::skip as designed.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added`; `marley_project.md` FileTree section; selftest README cwd/
  rebuild gotchas.
- **Tooling fix:** `bundle-app.sh` now ALWAYS rebuilds (was gated on `[ ! -x $BIN ]` → stale binary);
  added `clickat:` to `drive.swift`.
- **Knowledge:** PR `PR-claude-selftest-rebuild-and-cwd-before-blaming-code-001` (stale-binary + .app-
  cwd=/ traps); failure `BF-claude-toggle-tests-too-shallow-missed-recurse-guard-mutants` (the MSI 82.6
  survivor). aar-submit `completed` (5).
- **Ticket:** forge #56 → done; local doc → closed/; pair archived. **4/6 of M2.A.** First UI ticket
  fully self-test-verified (render capture + drive-click).
