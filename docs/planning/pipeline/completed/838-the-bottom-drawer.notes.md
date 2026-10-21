# The bottom drawer: Problems and Debug — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-838-the-bottom-drawer.md
- **Pipeline spec:** 838-the-bottom-drawer.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #837; Chad's "lets continue on" (2026-10-21).
- **Classification / tier:** feature, L. Marley crate (`marley_workbench`), two small Zed hunks
  (`workspace/src/dock.rs`, `diagnostics/src/diagnostics.rs`).
- **Recall (§18.3):** AD-claude-774 (Debug keeps the bottom dock; its button at the left of the
  status bar), L-claude-707 (an opened dock takes the focus), PR-claude-773. The brain
  (consultation cd6086e7): nothing on this seam.
- **Discovery:** `Dock::render` (dock.rs:1278; the panel's view at ~1368), `Dock::panel::<T>`
  (:521), `panel_index_for_persistent_name` (:533), `activate_panel` (:946), `active_panel`
  (:978); `ProjectDiagnosticsEditor::new` (diagnostics.rs:168, private), `deploy` (:386);
  `DiagnosticIndicator`'s click (items.rs:123) calls `deploy`; the debugger panel's name
  `DebugPanel` (debugger_panel.rs:1514); the board `BottomPanel.dc.html`.
- **Decisions:** D1 to D3 in the spec.

### Design
- **Zed hunk, dock.rs:** `pub struct MarleyDockHeader(pub fn(&Dock, WeakEntity<Dock>, &mut Window,
  &mut App) -> Option<AnyElement>)`, a `Global`; `Dock::render` draws its element above the panel
  when it answers one. Ledger row.
- **Zed hunk, diagnostics.rs:** `new` made `pub`; `pub struct MarleyDiagnosticsDeploy(pub
  fn(&mut Workspace, &mut Window, &mut Context<Workspace>) -> bool)`, a `Global`; `deploy` returns
  early when it answers true. Ledger row.
- **`drawer.rs` (new, Marley):** `ProblemsPanel { diagnostics: Entity<ProjectDiagnosticsEditor>,
  focus, position }`, a bottom `Panel` (persistent name `MarleyProblemsPanel`, icon `Warning`, no
  status-bar button: the indicator is its button). `init`: on each new workspace, add the panel;
  set `MarleyDiagnosticsDeploy` (Marley layout: `focus_panel::<ProblemsPanel>`, true; else false)
  and `MarleyDockHeader` (bottom dock, Marley layout, active panel `MarleyProblemsPanel` or
  `DebugPanel`: the strip; else none). The strip: `Button`s "Problems N" (N from the project's
  `diagnostic_summary`) and "Debug" with `toggle_state`; a click `activate_panel` by name.
- **File manifest:** `drawer.rs` (new), `marley_workbench.rs` (mod, init), `Cargo.toml`
  (`diagnostics` dependency), `crates/workspace/src/dock.rs`, `crates/diagnostics/src/diagnostics.rs`
  (Zed, ledger rows), the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002 | crate with a syntax error, main.rs open; the bottom row's count | `838-01-problems` |
| REQ-002 | the Debug tab | `838-02-debug` |
| REQ-003 | `debugger: start`, the GDB config | `838-03-session` |
| REQ-001/002 | Marley Day, the Problems tab | `838-04-day` |

### Risks
- rust-analyzer in the scenario: its check builds into `CARGO_TARGET_DIR`; the scenario points it
  at `$E2E_WORK/target`, never the shared target.
- GDB's DAP needs gdb ≥ 14 (the box has 17.2).

## Phase 2 — Code
- **Built:** the two Zed hooks (`MarleyDockHeader` in `Dock::render`, computed before the resize
  handle's closure holds `cx`, and the panel's box `flex_1 min_h_0` so it takes what the strip
  leaves; `MarleyDiagnosticsDeploy` in `deploy`, and `ProjectDiagnosticsEditor` and its `new`
  public), ledger rows first; `drawer.rs` as designed; `ToggleProblems`; the `diagnostics`
  dependency.
- **Deviation:** the struct itself was `pub(crate)`, so it is public too (the same ledger row). The
  hook passes the dock's weak handle by reference (clippy's needless pass by value).
- **Review:** the hooks copy their function pointer out of the global before calling it, since the
  call takes `cx` mutably; `tabs` reads the Problems panel during the dock's render, which nothing
  updates then.
- **Gate:** `just gate-diff` green (17 gates and the receipt).


## Phase 3 — Test
- Probe: the bottom row's count sits at (452, 985) beside the file and branch, not where #837's
  shot had it; the drawer's tabs at (307, 685) and (376, 685). The GDB session started from
  `.zed/debug.json` and paused at `main`, and the dock switched to Debug by itself.
- Recorded run (`script/e2e/838-the-bottom-drawer.sh`, sway, the gated build):
  - `838-01-problems`: the bottom dock with "Problems 3" lit and "Debug"; Zed's diagnostics under
    them: main.rs, the syntax errors at line 3; no Problems center tab (REQ-001, REQ-002).
  - `838-02-debug`: Debug lit; the debugger panel's empty state (No Breakpoints Set, New Session,
    Edit debug.json) under the same tabs (REQ-002).
  - `838-03-session`: `debugger: start`, Debug hello: hello.c open at `main`, the frame `main`,
    GDB's console, the session's controls, Debug lit (REQ-003).
  - `838-04-day`: Marley Day, Problems lit with its list (REQ-001, REQ-002).
- Not shot: Zed's layout (the hooks answer none there: read in the diff).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added), the guide's bottom drawer bullet, the architecture note's "The
  bottom drawer", the look plan's W3.2; two ledger rows (dock.rs extended, diagnostics.rs new).
- **Knowledge:** AD-claude-838-the-drawer-is-the-bottom-dock-with-a-strip-001,
  L-claude-838-gdb-and-rust-analyzer-make-a-scenario-001.
- **Brain:** decision on consultation cd6086e7.
- Ticket closed, pipeline archived.
