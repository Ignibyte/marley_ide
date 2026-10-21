# Files pops out from the left — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-868-files-pop-out-from-the-left.md
- **Pipeline spec:** 868-files-pop-out-from-the-left.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-21, mid-#838: Files as a left-hand nav item that pops out, not a
  center tab. Filed as #868 at the Queue's top; his pick between a pushing dock and an overlay is
  asked, the dock the default.
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** L-claude-707, AD-claude-773, AD-claude-837. The brain (consultation
  0da97eab): nothing on this seam.
- **Discovery:** `panel_tabs::MOVED` (:39), `inspector::INSPECTED`, `apply_defaults`
  (marley_workbench.rs:~1520) and `LayoutState` (:1033), `rail.rs::render_header` (:4820),
  `default.json`'s `project_panel.dock: "right"` (:889).
- **Decisions:** D1, D2 in the spec.

### Design
- `LayoutState.zed_project_panel_dock` kept from the defaults; `apply_defaults` sets
  `project_panel.dock` to `Left` in the Marley layout and back in Zed's (`DockSide`).
- `MOVED` drops "Project Panel"; the settings observer's first pass closes a Files tab left from
  before (`close_tabs` on the name). `INSPECTED` drops it; the Inspector's tests of membership
  follow.
- `render_header`: `IconButton::new("marley-rail-files", IconName::FileTree)`, tooltip "Files",
  toggled while the left dock is open on the project panel, dispatching
  `project_panel::ToggleFocus` built by name.
- **File manifest:** `marley_workbench.rs`, `panel_tabs.rs`, `inspector.rs`, `rail.rs`; the
  scenario; docs for #747 and #837 reworded.

### Visual check plan
As the spec's UI proof: five shots.

### Risks
- A saved workspace with a Files tab (#773's restore) must not bring it back: the restore refuses
  a name outside `MOVED`, and Zed's cleanup drops the row.

## Phase 2 — Code
- **Built:** `LayoutState.zed_project_panel_dock` and `zed_project_panel_starts_open`;
  `apply_defaults` puts the panel on the left and closed at start in the Marley layout; "Project
  Panel" out of `MOVED` and `INSPECTED` (their comments and the Inspector's module doc follow);
  `rail.rs::render_files_button` before Add Project, lit while the left dock shows the panel.
- **Deviation:** the first probe showed the tree open at start (Zed's `starts_open` for a new
  project); a pop-out starts closed, so the Marley layout turns `starts_open` off too.
- **Review:** the button reads the workspace through the rail's own `MultiWorkspace` handle during
  the rail's render (nothing updates it then); the action is built by name, its error logged.
- **Gate:** `just gate-diff` green (17 gates and the receipt).


## Phase 3 — Test
- Scenario `script/e2e/868-files-pop-out-from-the-left.sh` (sway), the gated build; the probe put
  the Files button at (196, 17).
- `868-01-closed`: lib.rs open, no tree; the rail's header: PROJECTS THREADS SESSIONS, then the
  Files button (unlit), Add Project, Shrink (REQ-003).
- `868-02-open`: the button lit; the tree (repo, src, lib.rs, README.md) between the rail and the
  editor, the editor in view, no Files tab (REQ-001, REQ-003).
- `868-03-key`: put away, then Ctrl+Shift+E: the tree back in the left dock, no tab (REQ-002).
- `868-04-inspector`: the setting on, `git panel: toggle focus`: the Inspector with Changes and
  Outline only (REQ-004).
- `868-05-day`: Marley Day, the tree out beside the rail (REQ-001).
- The tree opens about 570 px wide in the test profile: Zed keeps each panel's size in the
  profile's key-value store, and that profile carries an earlier scenario's; a fresh one opens at
  `default_width` (240). Not in scope.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Changed), the guide (the layout table, Side panels as tabs, Files pops out,
  the Inspector, the keys table), the architecture note's "Files at the left", no Zed path.
- **Knowledge:** AD-claude-868-files-is-a-left-dock-pop-out-001.
- **Brain:** decision on consultation 0da97eab.
- Ticket closed, pipeline archived. Chad's pick between the dock and an overlay is still open; the
  dock shipped as the default.
