---
pipeline_id: b351be7f-67f0-40f4-bebe-bf6fffb696b5
ticket: docs/planning/tickets/open/TICKET-923-side-panels-as-the-board.md
status: Phase 4 — Complete PASS
title: The Inspector and Files as the SidePanels board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/D-panels-editor.md, /srv/stacks/marley/docs/design/marley-canvas-2026-10-20/project/SidePanels.dc.html]
---

## Title
The Inspector and the Files column as the SidePanels board draws them. The Inspector's switcher
becomes a segmented track, its Changes tab carries a count, and an Inspector button at the right
of the tab bar shows it. Files gets a "Find a file" field that narrows the tree, an "M" on each
changed file, and opens 320 px wide.

## Scope
### In
- `inspector.rs` (Marley crate): the switcher in `Inspector::render` drawn as a segmented track:
  a rounded track holding one equal segment per panel, the chosen one filled. Today it is a row of
  Subtle buttons over a bottom border.
- `inspector.rs` (Marley crate): "Changes N" on the Changes segment, N being the active
  repository's changed files, kept current from the project's `GitStore` events. "Changes" alone
  when nothing changed or there is no repository.
- `pane.rs` (Zed crate hunk): `MarleyTabBarEnd`, a global `default_render_tab_bar_buttons` asks
  for an element to draw before its + button. Without the global, Zed's buttons are unchanged.
- `inspector.rs` and `marley_workbench.rs` (Marley crate): the hook's answer in the Marley
  layout, an "Inspector" button with a panel icon. It is lit while `marley.side_inspector` is on.
  A click runs `marley: toggle inspector`. Turning it on from the button also opens the Inspector
  on Changes in that window.
- `project_panel.rs` (Zed crate hunk): `MarleyFilesFilter`, a global the panel asks when it is
  made. While it is set, the panel draws a single-line "Find a file" field with a magnifier above
  its list. While the field holds text, `update_visible_entries` lists only the files whose
  worktree-relative path contains it, with their folders open. Emptying the field, or Escape in
  it, gives back the tree as it was.
- `marley_workbench.rs` (Marley crate): `apply_defaults` and `LayoutState` give the Marley layout
  two more project-panel defaults, as #868 did for its dock: `git_status_indicator` on, so a
  changed file shows Zed's own mono "M" (or A, D, U, !) at its row's right, and `default_width`
  320. Zed's layout gets Zed's values back.
- The left dock's first open in the Marley layout at 320 px. Phase 2 finds why the review's tour
  opened it about 570 px wide, likely the left dock's shared size (`resize_all_panels_in_dock:
  ["left"]`) or a stored `dock_panel_size`. A width the user drags still wins afterwards.

### Out (explicitly deferred)
- The Changes panel as the board lays it out: the "Changes on main" heading with Stage All,
  checkboxes at the left of each row, the full-width coat-orange Commit, and "Ask an agent" beside
  Commit and Publish. That is an M on its own in `git_ui/src/git_panel.rs`. **A follow-up ticket**
  (the review's "Changes panel layout differs", look, and "No Ask an agent button by Commit",
  missing).
- Outline's "Outline · main.rs" header and a kind label on every symbol: the same follow-up
  ticket (the review's "Outline lacks its header and kind labels", look).
- The breadcrumb "repo › src › main.rs › fn main" and its "+N since main" chip: the same follow-up
  ticket (the review's "Breadcrumb format differs", look, and "No +2 since main in the
  breadcrumb", missing).
- Files inside the Inspector: deferred by #868, Chad's request that Files be a left pop-out.
- The Inspector off by default: kept as is (#837, D3).
- The Inspector restored after a restart: deferred by #837.
- "Git Panel" and "Outline Panel" as tab titles: build order line 1, its own ticket.

## Reference (§20)
Upstream Zed: the project panel (`project_panel`), its git status badge (`git_status_indicator`,
`git_status_indicator()`) and its `default_width`; the pane's tab bar buttons
(`default_render_tab_bar_buttons`); the git store's repository status (`project::git_store`).
Kept: the panel's tree, its keys and its selection; the badge Zed draws; the tab bar's +, split
and zoom, and its rule that the buttons show on the focused pane. Marley adds the filter field,
the Inspector button and the switcher's look, each asked of a Marley global so Zed's layout stays
Zed's. The Inspector is Marley's (#837).

### Prior art
- `crates/marley_workbench/src/inspector.rs`: `INSPECTED` (:24), `WIDTH` 320 (:27),
  `Inspector::new` (:122), the switcher of Subtle buttons (:247). It holds the workspace weakly
  and no project.
- `crates/marley_workbench/src/marley_workbench.rs`: `ToggleInspector` (:207, :1237),
  `toggle_inspector` (:1438) writing `marley.side_inspector`; `SidePanels` (:676); `LayoutState`
  (:1062) and `apply_defaults` (:1570), where #868 sets the project panel's dock and
  `starts_open` per layout.
- `crates/workspace/src/pane.rs`: `default_render_tab_bar_buttons` (:4472) returns early when the
  pane has no focus (:4477) and builds +, split and zoom inline. Marley's hooks there,
  `MarleyNewItemMenu` (:4420) and `MarleySplitPictures` (:4362), are the pattern. #699 rejected
  `set_render_tab_bar_buttons` (per pane, copies the buttons).
  `docs/zed_architecture/subsystems/07-workspace-panes-palette.md:135` names that closure.
- `crates/ui/src/components/button/toggle_button.rs:171`: `ToggleButtonGroup`. Its chosen entry
  is accent-tinted, its width is its parent's, and its column count is a const. The board's
  track is a filled segment, not an accent. L-claude-660 and L-claude-664 record its traps.
- `crates/project/src/git_store.rs`: `Repository::status` (:6374), `status_summary` (:6378).
  `diff_header.rs:56` reads `active_repository`; `rail.rs:51` subscribes to `GitStoreEvent`.
- `crates/project_panel/src/project_panel.rs`: `git_indicator` from the `git_status_indicator`
  setting (:6214), `git_status_indicator()` (:8393: M, A, D, U, !), drawn at a file row's right
  and as a dot on a folder (:6669). `assets/settings/default.json:991` has it off. Marley sets
  neither it nor `default_width` (grep in `crates/marley_workbench/src`: none).
- `project_panel.rs`: no filter exists (grep "filter": only diagnostics and iterator filters).
  `update_visible_entries` (:4605) lists expanded folders only. `filename_editor` (:812) is the
  panel's single-line editor pattern. The `file_finder` and `fuzzy` crates own path matching.
- Width: `assets/settings/default.json:889` (`default_width` 240), `:880`
  (`resize_all_panels_in_dock: ["left"]`); `crates/workspace/src/dock.rs:395`
  (`PANEL_SIZE_STATE_KEY`, "dock_panel_size") and `:1174`.
- #837's spec (the Inspector, its count left out) and #868's (Files as a left pop-out).
  AD-claude-837, AD-claude-868.
- The behavior maps: `docs/zed_architecture/` has no project panel filter. Warp's: not this seam.

## UI proof
Scenario `script/e2e/923-side-panels-as-the-board.sh` (sway). A scratch repository with
`src/main.rs`, `src/board.rs` and `README.md` committed, then `src/main.rs` and `README.md`
changed. The run keeps the copied profile's database, so a stored left-dock size is in play as in
the review's tour. No agent CLI is needed.
- `923-01-button`: `main.rs` open, the pane focused: the tab bar's Inspector button, unlit
  (REQ-001).
- `923-02-inspector`: the button clicked: the Inspector at the right, the button lit, the
  segmented switcher with Changes chosen and reading "Changes 2" (REQ-001, REQ-002, REQ-003).
- `923-03-count`: `src/board.rs` changed on disk: "Changes 3" (REQ-003).
- `923-04-off`: the lit button clicked: the Inspector gone, the button unlit (REQ-004).
- `923-05-files`: `project panel: toggle focus`: the tree about 320 px wide, the "Find a file" field at
  its top, "M" at the right of `main.rs` and `README.md` (REQ-005, REQ-006).
- `923-06-filtered`: "board" typed in the field: only `src` and `board.rs` (REQ-007).
- `923-07-cleared`: Escape in the field: the full tree as before (REQ-007).
- `923-08-zed`: Zed's layout: the tab bar's +, split and zoom alone (REQ-008).
- `923-09-day`: Marley Day, the Marley layout, the Inspector and Files both open (REQ-001,
  REQ-002, REQ-006).

## Locked-In Decisions
- D1 — One slice: the Inspector and Files. Files is the panel used most (Chad's #868), and the
  Inspector's three parts finish #837's column. The Changes panel's layout, Outline's header and
  the breadcrumb go to a follow-up ticket.
- D2 — The switcher is Marley's own segmented track in `inspector.rs`: a rounded track, equal
  segments, the chosen one filled with the theme's selected element colour, the others in muted
  text. Not `ToggleButtonGroup`, whose chosen entry is accent-tinted.
- D3 — The count is the number of entries in the active repository's status. The Inspector keeps
  the workspace's project from when it is made and subscribes to its `GitStore`. It never reads
  the workspace for this in `render`. Zero or no repository draws "Changes".
- D4 — The button comes through a new hook in `default_render_tab_bar_buttons`, drawn before +.
  As +, split and zoom do, it shows on the focused pane only. It shows only in the Marley layout,
  where the Inspector exists.
- D5 — The button is the setting: lit while `marley.side_inspector` is on, and a click runs
  `marley: toggle inspector`. Turning it on from the button also opens the Inspector on Changes,
  so the click shows something. Turning it off closes every Inspector, as #837 does.
- D6 — The filter matches the typed text as a case-insensitive substring of a file's
  worktree-relative path, not fuzzily, so the tree never shows a file the text does not name. It
  follows the panel's own hiding rules (`hide_gitignore`, `hide_hidden`). It changes no folder's
  open state: emptying the field gives back the tree as it was.
- D7 — The "M" is Zed's own `git_status_indicator`, turned on as a Marley-layout default beside
  #868's dock default. A folder keeps Zed's dot. A user's own value for the key wins.
- D8 — Files opens at 320 px, the Inspector's own width. A drag still sets it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Marley layout is on, the focused pane's tab bar shall show an Inspector button, lit while the Inspector is on, in Marley Night and Marley Day. | `923-01-button`, `923-02-inspector`, `923-09-day` |
| REQ-002 | WHEN the Inspector shows, its switcher shall be a rounded track of equal segments with the chosen one filled. | `923-02-inspector`, `923-09-day` |
| REQ-003 | WHILE the active repository has changed files, the switcher's Changes segment shall read "Changes N" with their number, and N shall follow a change on disk. | `923-02-inspector`, `923-03-count` |
| REQ-004 | WHEN the user clicks the lit Inspector button, Marley shall close the Inspector and unlight the button. | `923-04-off` |
| REQ-005 | WHEN Files opens in the Marley layout, its column shall be about 320 px wide. | `923-05-files` |
| REQ-006 | WHILE the Marley layout is on, Files shall show a "Find a file" field at its top and a status letter at the right of each changed file's row. | `923-05-files`, `923-09-day` |
| REQ-007 | WHILE the filter field holds text, Files shall list only the files whose path contains it, with their folders; WHEN the field is emptied, it shall show the tree as it was. | `923-06-filtered`, `923-07-cleared` |
| REQ-008 | WHILE Zed's layout is on, the tab bar and Files shall be Zed's: no Inspector button, no filter field. | `923-08-zed` |
| REQ-009 | Each Zed-crate hunk shall carry a `// Marley:` comment and a row in `docs/marley/zed-touchpoints.md`. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the switcher, the count and its subscription; the tab-bar hook and the button;
  the project panel's filter hook and field; the two layout defaults and the first-open width;
  the touchpoint rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
