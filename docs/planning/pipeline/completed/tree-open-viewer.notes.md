# open a file from the tree into the viewer — Notes

- **Forge ticket:** #98 `b3be0115-7d23-43ef-b115-51ebb7728260` · **AAR:** `aa68318f-2216-464a-8bec-efd5ecb96fff`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-098-tree-open-viewer.md

## Phase 1 — Plan
- **Request:** forge #98 (M4 2/10) — the file-tree ⌘-click open + a shared helper + a dir guard.
- **Pre-flight:** the tree file-row on_mouse_down (app.rs ~1663) inserts the path (#59); `row.is_dir` +
  `file_tree.path_at(index)`; #97's finder ⌘↵ open is the pattern to refactor onto the shared helper.
- **Decisions:** D1 ⌘-click opens / plain inserts; D2 read-on-Ok shared helper; D3 viewer_open_path dir-guard.
- **AAR id:** `aa68318f-2216-464a-8bec-efd5ecb96fff`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **code_view.rs (PURE):** `use std::path::Path` (add); `pub fn viewer_open_path(path: &Path, is_dir: bool) -> Option<PathBuf>` = `if is_dir { None } else { Some(path.to_path_buf()) }`.
- **app.rs SHIM:** `#[cfg_attr(test, mutants::skip)] fn open_file_in_viewer(&mut self, path: PathBuf)` = read_to_string on Ok → code_view = Some(CodeViewState::new(...)). REFACTOR the #97 finder ⌘↵ block onto `if let Some(p)=viewer_open_path(&path,false){ self.open_file_in_viewer(p) }`. The tree FILE-row on_mouse_down (~1663): `_event`→`event`; `let path = path_at(index).map(to_path_buf)`; `if event.modifiers.platform { if let Some(p)=viewer_open_path(&path,false){ view.open_file_in_viewer(p) } } else { the #59 insert }`.
- **Mutation targets:** viewer_open_path is_dir branch.
- **Test plan:** `viewer_open_path_guards` (a dir→None; a file→Some(path)). cov/MSI 100. open_file_in_viewer + the tree/finder wiring masked (engine + #97 pattern; synthetic click blocked).
- **Risks:** ⌘-click a file = open (plain-click still inserts #59); the finder refactor must stay output-identical; open reads on Ok only (no crash on a bad read).

## Phase 3 — Implement
- **Built:** viewer_open_path (code_view.rs, dir→None/file→Some); app.rs `open_file_in_viewer` shared helper (read-on-Ok → CodeViewState::new); the #97 finder ⌘↵ refactored onto viewer_open_path+open_file_in_viewer; the tree FILE-row on_mouse_down now splits ⌘-click (open viewer) vs plain-click (#59 insert).
- **Verification:** fmt; check --all-targets 0 err; clippy OK; viewer_open_path test passes.

## Phase 3.5 — Inspect
- **Method:** self-review (a trivial pure guard + a masked shared helper + a masked click-split, reusing #97/#59 proven paths).
- **Lenses — no findings:** viewer_open_path (is_dir→None / else Some(path) — both tested); open_file_in_viewer reads on Ok only (a read error → viewer stays closed, no crash/unwrap; path→PathBuf owned before the mut borrow); the finder ⌘↵ refactor is output-identical (same read+new); the tree ⌘-click vs plain-click split preserves the #59 insert + the #71 is_command_running branch verbatim (viewer-open is a separate cooked-only affordance). No panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** viewer_open_path_guards (file→Some(path)/dir→None). `cargo nextest` → pass.
- **Self-test:** the ⌘-click tree open + finder ⌘↵ need synthetic input (ENV-BLOCKED); viewer_open_path is engine-tested (cov/MSI 100); open_file_in_viewer + the click-split are masked, sharing the proven #97 open + #59 insert paths.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #98 → done. **M4 2/10.** viewer_open_path (cov/MSI 100) + shared open_file_in_viewer + tree ⌘-click + finder refactor.
