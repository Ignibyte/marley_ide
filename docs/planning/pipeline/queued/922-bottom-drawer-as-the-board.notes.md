# The bottom drawer as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-922-bottom-drawer-as-the-board.md
- **Pipeline spec:** 922-bottom-drawer-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, line 9 (docs/marley/design-review-2026-10-23/README.md).
- **Classification / tier:** feature, L. Marley crate (`marley_workbench`) and one small Zed hunk
  (`crates/workspace/src/dock.rs`).
- **Recall (§18.3):**
  - AD-claude-838: the drawer is the bottom dock with a strip; it rejected a Problems list of
    Marley's own. This ticket supersedes that half (D1); Phase 4 writes the new AD.
  - AD-claude-774: Debug keeps the bottom dock by default. Move to the side is the user's move;
    the default stays (D3).
  - F-claude-865: a hook called from a render never reads that view. `tabs` and the shape hook run
    inside `Dock::render`; they read the `&Dock` they are given and the panels, never update the
    dock. Their clicks run later, outside the render.
  - F-claude-898: a toggle reads the dock's state, not the focus (D5).
  - PR-claude-773: a dock call tells two panels `set_active`; Move to the side moves both panels
    through the settings observer, which Zed already re-tells.
  - PR-claude-607: work done only while seen checks `dock.visible_panel()`. The Problems list
    refreshes on `DiagnosticsUpdated` whether seen or not, as the count does today; reading
    buffers only while seen is a choice for the Code phase if the refresh costs.
  - L-claude-838: rust-analyzer's syntax errors arrive within 25 s; GDB 17.2 speaks DAP; the
    scenario's target is `$E2E_WORK/target`.
  - F-claude-903 and PR-claude-687: the stand-in `claude` goes first on the terminals' PATH
    through the scenario's `.bashrc`.
  - L-claude-707: an opened dock takes the focus.
- **Discovery:**
  - `crates/marley_workbench/src/drawer.rs` (all of it; `ProblemsPanel` at `:126-235`, `tabs` at
    `:59-100`).
  - `crates/marley_workbench/src/send_selection.rs` `ask_to_fix` (`:225-284`).
  - `crates/workspace/src/dock.rs` `Dock::render` (`:1292-1406`).
  - Read only: `crates/diagnostics/src/diagnostics.rs` (`:351-394`, `:509-560`),
    `diagnostic_renderer.rs` `jump_to` (`:323`), `crates/debugger_ui/src/debugger_panel.rs`
    (`:1276`, `:1522-1548`, `:1592-1600`), `crates/dap/src/debugger_settings.rs`.
  - The board: `/srv/stacks/marley/docs/design/marley-canvas-2026-10-20/project/BottomPanel.dc.html`.
- **Decisions:** D1 to D7 in the spec.

### Design
- **`problems_panel.rs` (new, Marley crate).** `ProblemsPanel { project, workspace, filter:
  Entity<Editor>, problems: Vec<Problem>, refresh: Option<Task<()>>, zoomed: bool, focus }`;
  `Problem { path: ProjectPath, absolute_path: PathBuf, line: u32, severity: Severity, message:
  SharedString }` with `enum Severity { Error, Warning }`. On `DiagnosticsUpdated` (debounced as
  Zed's editor is), a spawned task opens each summarised path's buffer and collects primary
  entries up to the `include_warnings` bound. `render`: the filter field, then a `uniform_list` of
  rows; a row's click opens the path through `Workspace::open_path` and puts the caret on the line
  (as `jump_to` does); the pill calls `send_selection::ask_to_fix_at`. Position from
  `dap::DebuggerSettings::dock`, `set_position` writes `debugger.dock`, valid anywhere;
  `is_zoomed` / `set_zoomed` and a `ToggleZoom` handler emitting `ZoomIn` / `ZoomOut`. `count`
  stays for the pill.
- **`drawer.rs` (Marley crate).** `tabs` draws on any dock whose shown panel is Problems or Debug;
  each tab a label and a pill (`count_pill`); the Debug pill from the project's `DapStore`
  sessions (as `debug_row.rs` reads them); at the right three `IconButton`s with tooltips: Move to
  the side / to the bottom (`PanelHandle::set_position` on the shown panel), Maximize / Restore
  (focus the panel, dispatch `ToggleZoom`), Close (`Dock::set_open(false)`, then focus the
  center). A grab handle above the strip at the bottom. `shape`, the `MarleyDockShape` hook:
  `Card` for the dock showing the drawer in the Marley layout, else `Flush`.
- **`send_selection.rs` (Marley crate).** `ask_to_fix` keeps reading the caret and calls a new
  `ask_to_fix_at(workspace, path, line, problem, window, cx)`, which holds today's request and
  target picking.
- **`crates/workspace/src/dock.rs` (Zed crate).** `pub enum MarleyDockShape { Flush, Card }` and
  `pub struct MarleyDockShapeHook(pub fn(&Dock, &App) -> MarleyDockShape)`, a `Global`; asked in
  `Dock::render` beside `MarleyDockHeader`; `Card` swaps the side border for an all-round border,
  a 14 px radius and the inset (padding on the dock's wrapper or margin on the box, whichever the
  layout takes without overflow). `// Marley:` comments on the hunk.
- **`marley_workbench.rs`** (mod and init), **`Cargo.toml`** (`dap` dependency for
  `DebuggerSettings`).
- **Ledger:** `docs/marley/zed-touchpoints.md`, the `crates/workspace/src/dock.rs` row extended
  with `MarleyDockShape`: "Marley's bottom drawer is drawn as an inset rounded card (#922)"; to put
  back: re-add the hook in `Dock::render`. The `diagnostics.rs` row stays (`deploy` still opens the
  drawer); its "draws Zed's own diagnostics editor" wording changes to say the Marley layout lists
  them, and `ProjectDiagnosticsEditor::new` goes back to private if nothing else uses it.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001, 005, 009 | the crate with errors in two files; the drawer opened on Problems | `922-01-problems` |
| REQ-002 | a word from one error typed in the filter | `922-02-filtered` |
| REQ-003 | filter cleared, the second file's row clicked | `922-03-opened` |
| REQ-004 | the stand-in `claude` started in a terminal; a row's Ask an agent to fix | `922-04-asked` |
| REQ-005 | `debugger: start`, the GDB config, stopped at `main` | `922-05-paused` |
| REQ-006 | Maximize | `922-06-maximized` |
| REQ-006, 008 | Restore, then Move to the side | `922-07-side` |
| REQ-008 | Move to the bottom | `922-08-bottom` |
| REQ-007 | Close | `922-09-closed` |
| REQ-001, 005, 009 | Marley Day, the drawer on Problems | `922-10-day` |
| REQ-010 | not shot: the hooks answer `Flush` and none in Zed's layout | the review of the diff |

### Risks
- The inset inside the dock's fixed-size wrapper: a margin on a full-size box overflows in taffy.
  The Code phase checks the shot and moves the inset to padding if it clips.
- Moving the Debug panel to the side turns its panes' axes (Zed's own behaviour); the shot shows
  what that looks like on the right.
- Move to the side writes the user's `debugger.dock`; the scenario runs on a copy of the profile
  and moves the drawer back.
- Reading every summarised path's buffer opens buffers for files not open in an editor, as Zed's
  diagnostics editor does; a project with thousands of problems may need a cap on rows.
- Two Close buttons while Debug shows (the drawer's and the debugger strip's own) until the
  follow-up restyles the Debug tab.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
