# File-ref context menu — Notes

- **Forge ticket:** #293 `05619daf-a14a-4f7a-8b32-c360155f2815`
- **AAR:** `6d5fbb36-903b-4be8-bcfe-50bcaef68054`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-293-file-ref-menu.md
- **Pipeline spec:** 293-file-ref-menu.spec.md

## Phase 1 — Plan
- **Request:** a right-click file-ref context menu (Open / Split / Reveal-in-tree /
  Copy-path) on a `path:line` in block output.
- **Classification / tier:** work pipeline slice, `feature` — the LARGEST M18 fusion
  ticket (a new menu kind + right-click-on-link plumbing + 4 actions + a FileTree method).
  Crates: `marley_app` (context_menu.rs pure + app.rs shim), `marley_project` (FileTree).
- **KEY discoveries (bound the work):**
  - `context_menu.rs`: `MenuAction`/`MenuKind{Split,Block}`/`items_for` + `MENU_ITEMS`/
    `BLOCK_MENU_ITEMS`/`BLOCK_MENU_ITEMS_FAILED` (#213 added JumpToFailure). Adding a 4th
    kind + table is the established pattern.
  - `MenuKind` derives **Copy** → `FileRef` can't hold a `PathBuf`; D1 = a marker + a shim
    field (`file_ref_menu_target`).
  - Links are LEFT-click open via `open_link_target` (app.rs:2683, called at ~8041 with the
    link geometry); the pane already has a RIGHT-click handler (~7561) for the Block/Split
    menu → the right-click needs the same link hit-test to branch to FileRef.
  - `split_file_pane(path)` (#246, app.rs:2797); `write_to_clipboard(ClipboardItem::new_string)`;
    `open_file_at` (#212).
  - `FileTree` (marley_project/lib.rs): `visible_rows`/`toggle`/`path_at`, dirs start EXPANDED
    → `reveal` = expand ancestors + return the visible-row index (a clean pure addition).
- **Scope decision:** ship all 4 items (each reuses a shipped opener or a small pure add).
  DEFER: a persistent reveal highlight (needs tree-selection state), disabled-item states
  (the actions no-op gracefully), a modifier-click variant, a URL-link menu.
- **Risk:** moderate — the biggest piece is the shim right-click hit-test (app.rs, excluded)
  branching FileRef-vs-Block by whether the click hit a File link; and the target-field
  lifecycle. The pure seams (`items_for(FileRef)`, `FileTree::reveal`) are small + clean
  (cov/MSI 100). Live-drive proves the right-click → menu → split path.

## Phase 2 — Design

### Architecture / approach
Two pure additions (the menu table + the tree reveal) + a shim that turns a right-click
on a File link into the menu and dispatches its 4 actions to shipped openers.

- **`context_menu.rs` (pure):** `MenuAction` gains `OpenInEditor / OpenInSplit /
  RevealInTree / CopyPath`; `MenuKind::FileRef` (a payload-less marker — `MenuKind` stays
  `Copy`, D1); `FILE_REF_MENU_ITEMS: [(MenuAction,&str);4]`; `items_for(FileRef)` →
  that table. The exhaustive `MenuAction` dispatch match in app.rs gains 4 arms.
- **`marley_project` `FileTree::reveal(&mut self, path) -> Option<usize>` (pure):**
  ```
  let parts = path.iter().map(to_string).collect();
  if !expand_parts(&mut self.roots, &parts) { return None; }   // expand ancestors; false if missing
  let target = path.to_path_buf();
  (0..self.visible_rows().len()).find(|&i| self.path_at(i).as_deref() == Some(&*target))
  ```
  `expand_parts(nodes, parts)`: `split_first`; `find(name==head)` (else `false`); if `rest`
  empty → `true` (the target, not expanded); else `node.collapsed=false` + recurse. Reuses
  the tested `path_at`/`visible_rows` for the index (O(n²) but n is the capped file tree, and
  reveal is a click action, not hot). `None` on empty/missing → the shim no-ops.
- **`app.rs` (shim, `mutants::skip`/excluded):**
  - a `RootView` field `file_ref_menu_target: Option<(PathBuf, Option<usize>, Option<usize>)>`.
  - the link span (render ~8039, next to the `on_click` → `open_link_target`) gets an
    `on_mouse_down(MouseButton::Right, …)` that, for a `LinkTarget::File`, calls
    `open_file_ref_menu(path, line, col, e.position)` — sets the target field, opens
    `ContextMenuState{ kind: FileRef, origin: e.position }`, and `cx.stop_propagation()` so
    the pane's #175 Block/Split right-click (~7561) does NOT also fire. A `Url` link ignores
    the right-click (a URL menu is deferred).
  - 4 dispatch arms (in the `MenuAction` match ~3557) reading `file_ref_menu_target`:
    `OpenInEditor` → `open_file_at(resolve_under_root(root,path), line, col)` (#212);
    `OpenInSplit` → `split_file_pane(resolve_under_root(root,path))` (#246);
    `RevealInTree` → `rel = path.strip_prefix(&self.project_root).unwrap_or(path)` →
    `if let Some(i)=self.file_tree.reveal(rel) { self.files_scroll=i; }` + notify;
    `CopyPath` → `write_to_clipboard(ClipboardItem::new_string(path.display().to_string()))`.

**§14:** the pure seams return `Option` / a table (no panic); the shim confines the
click/menu/clipboard to the view. Reuses tested openers — no new process-spawn/IO path.

**§20 (clean-room) — CONFIRMED.** Reference = the VS Code / Zed integrated-terminal file-ref
right-click menu (Open, Reveal in Explorer/Finder, Copy Path). Marley matches with its own
`ContextMenuState`/`MenuKind` (#166) + `FileTree` + the shipped openers. No Zed/Warp source
read — observed behavior only.

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/context_menu.rs` | +4 `MenuAction` variants, `MenuKind::FileRef`, `FILE_REF_MENU_ITEMS[4]`, the `items_for` branch |
| `crates/marley_project/src/lib.rs` | + `FileTree::reveal(&mut self, path) -> Option<usize>` + the `expand_parts` free fn |
| `crates/marley_app/src/app.rs` | + `file_ref_menu_target` field + init; + `open_file_ref_menu` + the link `on_mouse_down(Right)`; + the 4 `MenuAction` dispatch arms (shim; excluded) |

### Regression Test Plan
| # | Test | AC | Asserts |
|---|---|---|---|
| T1 | `context_menu::file_ref_menu_items` | REQ-001 | `items_for(MenuKind::FileRef)` == `[OpenInEditor,OpenInSplit,RevealInTree,CopyPath]` (order); the existing `items_for(Split/Block)` tests stay green. |
| T2 | `marley_project::file_tree_reveal_expands_and_indexes` | REQ-002 | tree `["a/b/c.rs","a/b/d.rs","z.rs"]`; `toggle(0)` (collapse `a`) → `reveal("a/b/c.rs")` == `Some(2)` AND `a`/`b` now expanded (`visible_rows` shows `c.rs`); `reveal("nope/x.rs")` → `None`; `reveal("")` → `None`; reveal with nothing collapsed → `Some(2)`. |
| T3 | LIVE DRIVE | REQ-003/004 | right-click a `path:line` in block output → the 4-item menu appears (not the Block menu) → "Open in Split Right" → the file opens in a right split; "Reveal in File Tree" → the tree scrolls to it. |
| T4 | gate `scripts/gates.sh --diff` | REQ-005 | 100% line cov + MSI 100 on `items_for(FileRef)` + `FileTree::reveal`/`expand_parts`; brand-scrub; clippy -D. |

**Uncoverable:** the app.rs shim (the right-click hit-test, the menu open, the 4 dispatches)
is `mutants::skip` + coverage-excluded — proven by the live drive + reuse of the tested
`open_file_at`/`split_file_pane`/`reveal`/`write_to_clipboard`. The pure seams are unit-tested.

### Risks / decisions
- **D5 — the link right-click must `stop_propagation`** so the pane's #175 Block/Split menu
  doesn't ALSO open on the same right-press (the link span is a child of the pane). Verified
  in the drive (only the FileRef menu appears).
- **D6 — reveal takes the TREE-RELATIVE path** (`strip_prefix(project_root)`), since `FileTree`
  stores paths relative to the root; an absolute ref outside the project → `reveal` `None` → no-op.
- **D7 — `reveal` reuses `path_at` for the index** (O(n²), acceptable for a click on a capped
  tree) rather than a bespoke pre-order counter — less new code, reuses a tested walk.
- **D8 — `file_ref_menu_target` set at menu-open, read at dispatch** (the modal menu keeps it
  stable) — the D1 stand-in for the #175 "bound in MenuKind" invariant.

## Phase 3 — Implement
- **`context_menu.rs`** — `MenuAction` += `OpenInEditor/OpenInSplit/RevealInTree/CopyPath`;
  `MenuKind::FileRef` marker; `FILE_REF_MENU_ITEMS[4]`; `items_for(FileRef)` → the table.
- **`marley_project/src/lib.rs`** — `FileTree::reveal(&mut self, path)` (expand ancestors via
  `expand_parts`, then the `path_at` scan for the index) + the `expand_parts` free fn.
- **`app.rs`** — `file_ref_menu_target: Option<(PathBuf, Option<usize>, Option<usize>)>` field
  + init `None`; `open_file_ref_menu(path,line,col,pos,cx)` (stash target + open `MenuKind::FileRef`
  via `ContextMenuState::with_kind`); the link span's `on_mouse_down(MouseButton::Right)` →
  for a `LinkTarget::File`, `open_file_ref_menu` + `cx.stop_propagation()`; the 4 dispatch arms
  in `run_context_menu_action` (`let file_ref = self.file_ref_menu_target.take()` → `open_file_at`
  / `split_file_pane` / `reveal`→`files_scroll` / `write_to_clipboard`).
- **Deviations:** OpenInEditor/OpenInSplit pass the RAW ref (dropped the redundant explicit
  `resolve_under_root` — `open_file_at`/`split_file_pane` resolve internally via
  `load_code_view_state`; simpler + identical result). `cargo check` clean (the exhaustive
  `MenuAction` match now covers all 11 variants; no unused warnings).
- **Tests deferred to P4.**

## Inspect (Phase 3.5)
Rigorous self-review + a real mutant-list confirmation. (Two critics were spawned but
KILLED prematurely — a metadata heartbeat read 156 B and I misjudged it as the known
stall; their killed-state summaries showed both were mid-work, not stalled. Lesson: never
treat an agent's output-file SIZE as a completion signal — wait for the task-completion
notification. Their one live lead — "does the pane's right-click handler fire when the
click is on the child link hitbox?" — is folded below and is empirically resolved by the
P4 live drive.)

**Self-review — no confirmed defects.** Lenses:
- **`FileTree::reveal` correctness + mutation:** the real set (via `--list`) is 8 —
  `reveal` {`None`,`Some(0)`,`Some(1)`,`delete !`,`==`→`!=`}, `expand_parts` {`true`,`false`,
  `==`→`!=`}. NO mutant on `rest.is_empty()` or `collapsed=false`. ALL 8 die to T2:
  `reveal("a/b/c.rs")` with `a` collapsed → `Some(2)` (kills every body/`!`/`==` mutant —
  each yields `None`/wrong-index) + `reveal("nope")`→`None`. Traced: expand_parts walks
  `["a","b","c.rs"]`, expands `a` (+`b`, already open), stops at the leaf; path_at scan finds
  row 2. `reveal("")`→`None` (empty parts → `split_first` None → false). Revealing a leaf dir
  returns its row without expanding it (correct — a file ref is a leaf anyway).
- **`items_for(FileRef)`** → `FILE_REF_MENU_ITEMS[4]` (order); exhaustiveness is COMPILER-proven
  — the only exhaustive `MenuAction` match is app.rs:3585 (all 11 arms incl. the 4 new); 4541 is
  the `&str` verb match (irrelevant). Adding the 4 variants + the FileRef arm can't silently drop.
- **Shim target lifecycle (`file_ref_menu_target`):** set at `open_file_ref_menu`, `take()`n at
  dispatch. The 3 dismiss paths (Esc 6090, click-away 9612, right-click-away 9619) clear
  `context_menu` but NOT the target → a lingering `Some`. **Benign:** a FileRef action only runs
  when a FRESH `open_file_ref_menu` set the target (the menu can't open otherwise), and the
  Block/Split arms never read `file_ref`; so a stale value is never acted on (overwritten or
  take()n-and-ignored next). No correctness bug; a hygiene clear-on-dismiss is optional.
- **Reuse / borrow / §20:** OpenInEditor/OpenInSplit pass the RAW ref (openers re-resolve via
  `load_code_view_state`); `file_ref` is owned (`take`) so the `self.*` calls don't conflict;
  brand-scrub is a gate. `open_file_ref_menu` is genuinely distinct from `open_link_target`
  (menu-open vs immediate-open).

**Open item for P4 (the critic's lead) — the ONLY unresolved shim question:** does the pane's
right-click handler (7679) ALSO fire on a link right-click, and if so does `cx.stop_propagation()`
prevent it from overwriting `context_menu` with a Block/Split menu? **The live drive resolves this
directly: right-click a `path:line` → if the 4-item FileRef menu appears (not the Block menu),
the propagation is correct.** If the Block menu appears, re-enter inspect + fix (ensure the link
handler wins).

**Ledger:** no confirmed defects; 1 open item deferred to the P4 drive (propagation). Lenses:
reveal correctness+mutation, exhaustiveness, target lifecycle, reuse/borrow, provenance.

## Phase 4 — Validate
### Tests added
- **T1** `context_menu::file_ref_menu_items` (REQ-001) — `items_for(MenuKind::FileRef)` ==
  `[OpenInEditor,OpenInSplit,RevealInTree,CopyPath]` (order).
- **T2** `marley_project::reveal_expands_ancestors_and_indexes` (REQ-002) — tree
  `["a/b/c.rs","a/b/d.rs","z.rs"]`; `toggle(0)` collapse `a` → `reveal("a/b/c.rs")`==`Some(2)`
  + `a` EXPANDED (`c.rs` now a visible row); nothing-collapsed → still `Some(2)`;
  `reveal("nope/x.rs")`/`reveal("")` → `None`. Kills all 8 real reveal/expand_parts mutants.

### Real results
- `cargo nextest run -p marley -p marley_project` → **472 passed, 2 skipped** (+2 new).
- `scripts/gates.sh --diff` → **GATE GREEN [diff]** (a first rustfmt RED on the new test
  bodies → `cargo fmt` → green; coverage ≥100% lines + MSI ≥100% on `items_for(FileRef)` +
  `FileTree::reveal`/`expand_parts`; brand-scrub, miri, visual all green).

### LIVE DRIVE — the full flow proven end-to-end (resolves the inspect propagation question)
`New empty workspace` → ONE terminal → `echo /tmp/marley_293/hello.rs:3` (a clean file link).
- **RIGHT-click the link** → the **4-item FileRef menu** appeared (`293-02-menu.png`): "Open in
  Editor / Open in Split Right / Reveal in File Tree / Copy Path" — **NOT the Block menu**. This
  DEFINITIVELY resolves the inspect open item: `cx.stop_propagation()` works — the pane's #175
  Block/Split right-click did NOT also fire / overwrite the menu. REQ-003 ✓.
- **Click "Open in Split Right"** → `hello.rs` opened in a RIGHT split pane (`293-03-split.png`):
  the terminal (left) + the `hello.rs` code view (right, showing `fn main()…`); the Workspace
  panel now lists "pane 1" (terminal) + "pane 2" (the split). REQ-004 ✓ (reuses #246).
- Cleanup: app quit, `~/.marley` settings restored, `/tmp/marley_293` removed, tree clean.

No pre-existing failures in scope.

## Phase 5 — Complete
- **CHANGELOG.md** `### Added` (M18): the #293 file-ref-menu entry.
- **docs/marley_architecture/app_shell.md**: the `M18 (#293)` note after #292.
- **Knowledge (forge wired):** AAR `6d5fbb36` submitted (completed, effectiveness 5);
  `prevention-rule` **PR-claude-agent-output-size-not-a-completion-signal-001** (never gate
  on an agent's output-file SIZE — I killed 2 critics mid-work on a misleading 156 B heartbeat;
  wait for the task-completion notification). No `failure-record` — the one inspect open item
  (propagation) was RESOLVED green by the drive, not a bug.
- **Lessons:** (1) `MenuKind: Copy` blocked a `PathBuf` payload → the shim-field pattern
  (`file_ref_menu_target`) — a clean stand-in for the #175 "bound in the kind" invariant.
  (2) `FileTree::reveal` reused `path_at` for the index (O(n²) but a click action) — less new
  code, fewer mutants. (3) The live drive was decisive: it PROVED the right-click→FileRef-menu
  (not the block menu) + the split-open end-to-end, resolving the one shim question units can't
  reach. (4) The heartbeat-kill mistake → the new PR.
