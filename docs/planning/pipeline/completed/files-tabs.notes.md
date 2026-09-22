# M9 seq-5 — Files as a left panel + open-file-as-a-tab — Notes

- **Forge ticket:** #154 `2953a9bf-11fe-495a-a7d1-f022ae2d1e08` · **AAR:** `89d2ce11-b0ea-4e69-9788-fbe76889f7a0`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-154-files-tabs.md

## Phase 1 — Plan
- **Request:** forge #154 (M9 run 5/8) — Files → left panel; open-file → CodeView tab; retire the tiled panes.
- **Pre-flight:** today FileTree + CodeView are tiled panes (bodies in the pane `match kind`, app.rs FileTree
  ~2881-2938, CodeView ~2939-3050); open_files_pane (~1450) + open_file_in_viewer→open_code_pane (~1383/1405)
  open tiled panes. TabContent is Terminal|Cockpit.
- **Decisions:** D1 one CodeView tab reused (replace+switch); D2 Files = a LEFT panel toggled by 📁, scoped to
  active_project().root; D3 terminal_grid_index already guards a CodeView active tab (no grid → first terminal).
- **AAR id:** `89d2ce11-b0ea-4e69-9788-fbe76889f7a0`.

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
### Approach
PURE tabs.rs additions + a masked render refactor (two body-moves + center 3-branch + a left files strip).
- **tabs.rs (PURE, cov/MSI 100):**
  - `TabContent::CodeView(CodeViewState)` (import `crate::code_view::CodeViewState`, Clone/Eq).
  - `Tab::code(title, state)` ctor; `Tab::code_view(&self)->Option<&CodeViewState>` + `code_view_mut`.
  - `grid()`/`cockpit_section()` add a `TabContent::CodeView(_) => None` arm.
  - `Project::open_or_switch_code(&mut self, state)` — if a tab's code_view().is_some(): replace its content
    (`tabs[i].content = TabContent::CodeView(state)`) + `active=i`; else push `Tab::code(name, state)` + activate.
- **app.rs (SHIM masked):**
  - field `files_open: bool` (default false).
  - `fn code_view_body(&self, cv:&CodeViewState, colors)->gpui::Div` — MOVED from the CodeView pane arm
    (~2939-3050), but takes `cv` DIRECTLY (from the tab) not via pane_id/state.
  - `fn files_panel(&self, colors, cx)->gpui::Div` — MOVED from the FileTree pane arm (~2881-2938); self.file_tree.
  - CENTER render: `match active tab` → Terminal: the grid (existing) | Cockpit: cockpit_body | CodeView:
    code_view_body full-screen (all via `let active_cockpit`/`active_code` computed from self.shell before the
    else's workspace_mut). Reuse the empty-rect_list trick: non-terminal active tab ⇒ rect_list=Vec::new().
  - FILES STRIP: `let files_w = if self.files_open { FILES_PANEL_W(=240.0) } else { 0.0 };` shrink center_bounds
    (x+=files_w, w-=files_w) and render files_panel at the left strip when files_open.
  - `open_file_in_viewer` → `self.shell.active_project_mut().open_or_switch_code(state)` (drop open_code_pane).
  - "open-files" action (1818) + the 📁 icon (3649) → toggle `self.files_open` (drop open_files_pane).
  - RETIRE: open_files_pane, open_code_pane; the pane `match kind` FileTree/CodeView arms → minimal empty
    stubs (unreachable now — no pane gets those kinds; keeps the match exhaustive). PaneKind::FileTree/CodeView
    enum variants LEFT for a later cleanup (out of scope; noted).

### File manifest
- `crates/marley_app/src/tabs.rs` — PURE: CodeView variant, code_view accessors, open_or_switch_code + tests.
- `crates/marley_app/src/app.rs` — SHIM: files_open field; code_view_body + files_panel moves; center 3-branch
  + files strip; open-file→tab; open-files→toggle; retire the tiled panes.

### Regression Test Plan
| Test (tabs.rs, S=()) | AC |
|---|---|
| code_view_cases — a CodeView tab → Some(state); terminal/cockpit → None | REQ-001 |
| code_view_grid_cockpit_none — grid()/cockpit_section() None on a CodeView tab | REQ-002 |
| open_or_switch_code_cases — absent→append+active (state matches); existing→replace content + switch (no new tab; new state) | REQ-003 |
| DRIVEN: 📁 → a LEFT file panel (not a split) | REQ-004 |
| DRIVEN: open a file → a full-screen CodeView tab in the rail; back to terminal renders | REQ-005 |

### Risks
- Two masked render MOVES — the driven captures are the proof; also regression-check the terminal grid still
  tiles + the cockpit tab still renders (unchanged branches).
- CodeViewState must be constructible into a tab (it's Clone) — open_or_switch_code takes it by value.
- The files strip shrinks the center: confirm the tab content (terminal/cockpit/code) shifts right + nothing
  overlaps; the boundary/git-rect loops already no-op on empty rect_list.
- Pane-match FileTree/CodeView stubs are unreachable — note the PaneKind cleanup debt (a follow-up).

## Phase 3 — Implement
- **tabs.rs PURE:** TabContent::CodeView(CodeViewState); Tab::code ctor + code_view/code_view_mut; grid()/cockpit_section() add CodeView→None; Project::open_or_switch_code (replace existing code tab + switch, else append+activate; title=file name).
- **app.rs SHIM (masked):** field files_open; NEW code_view_body(cv,colors) + files_panel(colors,cx) methods (moved from the tiled CodeView/FileTree pane arms; code_view_body takes cv from the tab; files_panel plain-click opens a file). CENTER: a left Files strip (FILES_PANEL_W=240, shrinks center_bounds) when files_open; the active tab branches cockpit_body | code_view_body full-screen | grid (active_is_terminal → rect_list, else empty). open_file_in_viewer→open_or_switch_code; open-files action + 📁 icon→toggle files_open; DELETED open_files_pane+open_code_pane; the pane match FileTree/CodeView arms removed (Git→if-let); boot restore SKIPS FileTree/CodeView panes (retired).
- **Verify:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Phase 4 — Validate
- **Tests:** code_view_cases (REQ-001: code_view/code_view_mut Some/None), code_view_grid_cockpit_none (REQ-002), open_or_switch_code_cases (REQ-003: append/replace/switch + the "code" fallback title). 12 tab tests pass.
- **Self-test:** ft_files.png (📁 → Files panel on the LEFT showing the Marley tree; terminal shifts right — not a split; titlebar ~/.../Marley·main); ft_code.png (click complete.md → a CodeView tab fills the CENTER with a line-gutter, rail shows terminal 1[muted]/complete.md[active]). No panic.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first run GREEN; re-gated after the stale-comment fix, still GREEN). Both critics SUBSTANTIALLY CLEAN — only the stale comments (fixed); open_or_switch_code "untested" was pre-tests + is now cov/MSI 100.

## Inspect (Phase 3.5)
Method: 2 background critics (render-move fidelity + layout; pane-retirement + boot migration + panic) + my review + the driven captures. Both critics reviewed the final staged tree.

- **Critic A (render move) — NO FINDINGS** except one [nit] stale comment. Body fidelity faithful (files_panel + code_view_body byte-identical to the deleted arms; only intentional delta = plain-click opens a file, `_event` demoted); center math sound (the .max(0.0) clamp already present + region_widths clamps center); branch exclusivity exact (3-variant enum → mutually exclusive; rect_list empty for cockpit AND code); borrows sound; clippy 0.
- **Critic B (retirement) — SUBSTANTIALLY CLEAN.** Retirement complete (open_files_pane/open_code_pane deleted, zero prod callers; first_pane_of_kind/open_pane_rightmost stay alive via the Git path); migration self-healing (boot drops FileTree/CodeView, focus is PaneId(0) always the boot terminal, persist_grid re-saves without them on next mutation); panic-safe (a code tab's grid()→None → terminal_grid_index falls back; open_or_switch_code only replaces a code tab, never a terminal); open_or_switch_code correct.
- **[FIXED] stale comments (both critics):** app.rs:3633 (📁 icon "Reuses open_files_pane" → "toggles the LEFT Files panel") + app.rs:496 (boot "files/code/git session-less panes" → "FileTree/CodeView dropped on restore"). Cosmetic; corrected + re-gated.
- **[RESOLVED] open_or_switch_code untested (critic B [LOW]):** the critic reviewed before the tests landed — validate added code_view_cases/code_view_grid_cockpit_none/open_or_switch_code_cases; gate GREEN confirms cov/MSI 100.
- **[deferred → #159] close_tab last-terminal panic (both, [INFO]):** pre-existing from #153 (cockpit tabs), widened by code tabs; unreachable (close_tab unwired). Already filed #159.
- **[deferred → #159] PaneContent::FileTree/CodeView vestigial:** test-only construction now (clippy clean); enum cleanup folded into #159.

Lenses: render fidelity, layout math, branch exclusivity, borrows, retirement completeness, migration safety, panic-safety. No blocking findings.

## Phase 5 — Complete
- CHANGELOG + app_shell seq-5 note; forge #154 → done. **M9 5/8.** Files=LEFT panel; open-file=full-screen CodeView tab; tiled file/code panes retired. LESSON: a body-MOVE leaves stale comments at the OLD sites (grep the deleted fn name); boot must DROP retired pane kinds from a persisted grid (self-healing).
