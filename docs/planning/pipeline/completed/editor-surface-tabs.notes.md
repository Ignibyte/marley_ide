# editor-surface-tabs — pipeline notes (forge #237, M13 sprint #26)

Pipeline: 4385bf63-1409-42ec-b23b-a0122982752c · AAR: 57916c99-669d-42de-ae86-b20adebe9d0a
Ticket: forge#237 (2d01685e-ac8a-4b73-a2c7-d4233b279b88). 10th (LAST) of the /work 228-237 train.

## Phase 1 / 2 — Plan + Design (discovery inline)

**chad #9 (anti-clutter):** files shouldn't clutter the rail; open under a single pane, as editor tabs.

**Discovery (Explore):**
- ALL file-opens funnel through `open_file_in_viewer(path)` (app.rs:2189) → `CodeViewState::new` →
  `Project::open_or_switch_code(state)` (tabs.rs:247, dedupe-by-path → a CodeView TAB). 5 entry points
  (⌘P ⌘↵ app.rs:1639; Files-tree click 2527; link 2182; search 2006; diff 6051). `rail_rows` emits a
  Tab row per tab → N files = N rail rows (the clutter).
- `CodeViewState { path, lines (pre-rendered), scroll }` (code_view.rs:143) — READ-ONLY, 1-file-per-tab;
  render `code_view_body(cv, colors)` (app.rs:2440) dispatched at app.rs:4637; only on_scroll_wheel.
- `open_or_switch_code` (tabs.rs:247) dedupes by path (found→replace+switch; absent→append+activate) —
  the exact pattern for the surface's internal open.
- `marley_editor::Buffer` (editor/src/buffer.rs) — ropey, EDITABLE (EditOrigin::Human|Agent) but TRAPPED
  in the prompt (TerminalPane.buffer); no file-load/save → editing is greenfield → DEFER (read-only v1).
- The pane grid CAN host a CodeView pane (workspace.rs) but the render was RETIRED (#154, app.rs:5358
  only draws Git) + split_focused always spawns a PTY → split-to-file needs reviving the pane render +
  an open_pane split path → DEFER (its own ticket).
- `TabContent { Terminal | Cockpit | CodeView(CodeViewState) }` (tabs.rs:21) + accessors (grid/
  cockpit_section/code_view) + Tab::code + persistence TabLayout::Code (app.rs:915/2148).

**Design (D1-D3):** recast the CodeView TAB to hold a multi-file `EditorSurface` (not a new variant —
reuse the render/persistence/rail; the PaneContent::CodeView pane variant is untouched, reserved for the
deferred split-to-file). Read-only (reuse CodeViewState + code_view_body). One editor surface per project.

- **PURE `editor_surface.rs`:** `EditorSurface { files: Vec<CodeViewState>, active: usize }` +
  `open(state)` [dedupe→switch+refresh / append+activate], `close(idx)` [remove + clamp; empties→caller
  drops the tab], `activate(idx)`, `active_file()`, `files()`, `len()`. Tests: open-appends,
  open-dedupes-to-existing, close-clamps, activate. cov/MSI 100.
- **tabs.rs:** `TabContent::CodeView(CodeViewState)` → `CodeView(EditorSurface)`; `Tab::code(title,
  surface)`; `code_view()`/`code_view_mut()` → the surface; a convenience `active_code_view()`; recast
  `open_or_switch_code(state)` → find/create the ONE editor tab + `surface.open(state)`.
- **app.rs:** `open_file_in_viewer` unchanged (still calls open_or_switch_code, now surface-routing);
  the CodeView render (4637) draws a FILE-TAB STRIP (name + × close + click-activate) above
  code_view_body(active_code_view()); persistence: serialize the editor tab's file paths (extend
  TabLayout::Code → a path list) OR skip (DEFER — decide at implement; keep bounded).

**Driven plan (control):** boot → open 3 files (Files-tree clicks) → ONE editor tab (1 rail row) with a
3-file tab strip (not 3 rail rows); switch/close a file tab.

**ENV:** chad granted control → driven captures on. After #237 → the train COMPLETE → report + offer push.
**SCOPE NOTE:** the biggest ticket; v1 = the surface + routing + strip (read-only). Editing +
split-to-file + (maybe) persistence are flagged follow-ups.

## Phase 3 — Implement
NEW `editor_surface.rs` (`EditorSurface { files: Vec<CodeViewState>, active }` + open/close/activate/
active_file/files/active_index + 3 tests). tabs.rs: recast `TabContent::CodeView(CodeViewState)` →
`CodeView(EditorSurface)`; Tab::code wraps; `code_view()`/`code_view_mut()` return the ACTIVE file (so
render+persistence unchanged); NEW `editor()`/`editor_mut()`; `open_or_switch_code` routes into the ONE
editor tab (`position(|t| t.editor().is_some())` → `surface.open(state)`, else create); rewrote
`open_or_switch_code_cases` for the surface behavior. app.rs: the file-tab STRIP render (name + × +
click-activate above `code_view_body`) + `close_editor_file(i)` (close the file; if the last, close the
editor tab via `close_tab_at`). `cargo fmt`/`check` clean; 329 tests pass (+3 editor_surface).

## Phase 3.5 — Inspect
1 critic (correctness/mutation/recast/render/persistence) + self-review + the DRIVEN capture. **1 HIGH
(fixed) + 2 LOW:**
- **[HIGH] MSI < 100** — `EditorSurface` had 4 surviving mutants (close's `>`→`==`/`<`/`>=`, activate's
  `<`→`<=`): the clamp/activate tests didn't DISTINGUISH the branches (only closed at the last index [the
  decrement + >=-clamp branches coincide]; activate(5) never hit idx==len). Code CORRECT, tests weak.
  FIXED: added an active>idx case, an active-MIDDLE case, and an activate(len) boundary → `cargo mutants
  -f editor_surface.rs` = **26 mutants: 23 caught, 3 unviable, 0 MISSED (MSI 100)**. Captured
  `BF-claude-clamp-branch-mutants-survive-non-distinguishing-tests` +
  `PR-claude-clamp-adjust-tests-must-distinguish-each-branch-001`.
- **[LOW] persistence degradation** — the editor tab now persists only its ACTIVE file (`code_view()` =
  active); the other open files are dropped on restart (was: N tabs, all survived). The explicit v1
  limitation (spec Out); NO crash (restore builds a valid 1-file surface). Noted in the CHANGELOG.
- **[LOW] VIEWER_ROWS unchanged** while the strip takes 28px — the bottom row can be marginally clipped;
  pre-existing constant, cosmetic. Left as-is (a follow-up polish).
- **CLEAN:** recast completeness (the only CodeView construction is Tab::code; PaneContent::CodeView is a
  DIFFERENT enum, untouched); no `active_file` OOB (the invariant holds across all ops); render borrows
  immutable; `close_editor_file` no-ops on a non-editor tab + close_tab_at is safe (non-terminal);
  0-workspace safe (after the launcher branch); clippy/clean-room clean.
**Phase 3.5 status: Inspect PASS.**

## Phase 4 — Validate

**Tests (all written at implement, run here):** `editor_surface` — `open_appends_then_dedupes`,
`close_clamps_and_signals_empty` (3 distinguishing clamp cases + OOR + last-file→false),
`activate_switches_in_range_only` (incl the `idx==len` boundary); `tabs::open_or_switch_code_cases`
(rewritten for the surface: N files → 1 tab, `files().len()` grows, dedupe switches). Two COVERAGE
tests added at validate (the recast exposed two never-tested arms): `tabs::code_view_cases` extended
with `editor()`/`editor_mut()` = Some on an editor tab, None on term/cockpit (line 129, the
`editor_mut` None arm, was shim-only); `workspace::states_mut_yields_every_pane` (the pump-all
accessor, only shim-called → a direct unit test). Both are legitimate untested-code fixes (§0
source-fix, NOT suppressions).

**Gate:** `git add -A && scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — coverage 100% lines
on editor_surface.rs / tabs.rs / workspace.rs (the two new tests closed the two uncovered arms the
recast introduced); MSI 100 (gate:5); clippy/fmt/docs/visual all green. 331 lib tests pass.

**Driven capture (control granted — REQ-002/003/004):** bundled + launched; opened 3 files
(`buffer.rs`, `lib.rs`, `movement.rs`). Captured `scratchpad/237-a-editor.png` and READ it —
confirmed: a horizontal FILE-TAB STRIP (`buffer.rs · lib.rs · movement.rs`, each with a × close,
`movement.rs` active) above the active file's `code_view_body`; the LEFT RAIL shows ONE editor row
("buffer.rs", the project's single editor tab) under the project — NOT 3 rail rows (the anti-clutter
goal, chad #9). The recast works end-to-end on the live app.

**Phase 4 status: Validate PASS — gate green [diff]; driven capture confirms the surface.**
