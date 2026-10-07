# Rusty's screens in the rail's header — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-672-rusty-screens-in-the-rail-header.md
- **Pipeline spec:** 672-rusty-screens-in-the-rail-header.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-06 items 2 and 3, and his explanation the same night (quoted in the
  spec); built 2026-10-07.
- **Classification / tier:** feature; `marley_workbench` only (`rail.rs`, `rusty/brain.rs`,
  `rusty/capture.rs`, the guide). No Zed crate.
- **Pre-flight:** green; #671 committed; cargo idle.
- **Recall (§18.3):**
  - #644: R-D9's header switch; the Brain view's fixed row; `ToggleBrainView`'s window handler.
  - AD-453: lists bind left and right only in the `menu` context (unchanged here).
  - The brain (consultation `2f3a43ea45e34a88a89207a5b5e55428`, recorded as
    `decisions/marley-keeps-its-top-tabs-…`): the Rusty group (#675) comes after this.
- **Discovery:** `render_header` (`rail.rs:4109`), `render_view_switch` (`:4159`),
  `render_new_page`, `render_add_project` (`:4543`), `render_filter` (`:4497`), `Rail::width`
  (`:126`, default 260, 180 to 600); `BrainView::render_fixed_row` (`brain.rs:1008`),
  `render_search` (`:1061`), `today` (`:629`) and the six `open_*` (`:382-433`);
  `capture::open_today` (`capture.rs:95`); `ui::TabBar` (`tab_bar.rs:100-130`: the bottom line is
  inside `Tab::container_height`); `IconButtonShape::Square` (`icon_button.rs:260`: width and height
  `IconSize::square`); at Chad's 24px UI font a Small square is about 26px, so ten need about 300px.

### Design
- **`rusty/brain.rs`**: `pub(crate) enum Screen { Today, Graph, Tasks, Decisions, Memory, Skills,
  Secrets }` with `ALL`, `id` (the old element ids), `icon`, `label`; `pub(crate) fn
  open_screen(screen, multi_workspace, brain, window, cx)`: Today through the Brain view's `today`
  when the view exists (it reveals the page in the tree), else `capture::open_today` in the shown
  workspace; the rest through each tab's `open_later`. `render_fixed_row` and the six `open_*`
  methods go. `render_search` takes `Tab::container_height` in place of `py_2`.
- **`rusty/capture.rs`**: `open_today` becomes `pub(super)`.
- **`rail.rs`**: `render_header` takes the square's size, computed in `render` (it needs `&mut
  Window`). `render_view_switch` becomes the header's group: Projects, Brain, the screens that fit,
  and a `…` `PopoverMenu` with a `ContextMenu` of the rest (label and icon each). Every header
  button is `IconButtonShape::Square`. `screens_that_fit(room, button, gap)`: all seven when nine
  buttons fit, else as many as fit beside Projects, Brain and `…`. The group is `min_w_0` and
  `overflow_hidden`, so window controls on the left (not in the sum) can only clip the screens,
  never the `+`. `render_filter` takes `Tab::container_height`.
- **Guide**: the Brain view article and the rail header.
- **File manifest:** `rail.rs`, `rusty/brain.rs`, `rusty/capture.rs` (Marley crate),
  `guide/index.html`, the scenario; docs.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001, 003, 005 | Rusty's stand-in on; a project with a terminal | `672-01-projects` |
| REQ-005 | types in the filter | `672-02-filtering` |
| REQ-001, 004, 005 | clicks Brain | `672-03-brain` |
| REQ-002 | clicks Projects, then the Graph button | `672-04-graph` |
| REQ-003 | clicks `…` | `672-05-overflow` |
| REQ-003 | drags the rail's edge wider | `672-06-wide` |

### Risks
- The sum leaves out Linux window controls drawn in the header; with them, the group clips the
  last screens rather than pushing the `+` off. Hyprland and sway draw none.
- Square buttons are a little smaller than the old Default ones at the same icon size.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the ticket in progress; the backlog row removed.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-07)
### Built
- **`rusty/brain.rs`**: `Screen` (`ALL`, `id`, `icon`, `label`) and `open_screen`; the six `open_*`
  methods and `render_fixed_row` removed; `render_search` at `Tab::container_height`.
- **`rusty/capture.rs`**: `open_today` is `pub(super)`.
- **`rail.rs`**: `render` computes `IconSize::Small.square` once and hands it to `render_header`;
  `screens_that_fit` (a loop, no casts, for the crate's lint table); `render_view_switch` draws
  Projects, Brain, the screens that fit and `render_more_screens` (`…`, a `PopoverMenu` over a
  `ContextMenu` of the rest); `Rail::open_screen`; every header button square; `render_filter` at
  `Tab::container_height`.
- **The header's height**: the title bar's plus 1px. The exploratory run's pixel read put the rail's
  lines at rows 41 and 89 and the main column's at 42 and 90: the window draws a 1px line under the
  title bar that the header, at the title bar's height with its border inside, did not allow for.
  A pre-existing 1px offset, fixed here because Chad's ask is that the lines meet.
- **The guide**: the Brain view article names the screens in the header and `…`.
- **The scenario**: `lines_meet` reads the border rows at x 150 and x 700 from each shot.

### Deviations
- The header grows by 1px (above); not in the plan, needed for REQ-005 to hold.

### Review
- `open_screen` reads the multi-workspace and updates the Brain view or the workspace from a rail
  listener, never the rail itself, so nothing is updated while it is being updated.
- A window with Linux window controls in the header: the sum leaves them out, and the group, being
  `min_w_0` and `overflow_hidden`, clips its last buttons first; the `+` stays.
- Rusty not connected: the header shows PROJECTS as before; `render_view_switch` is not called.

### Gate
`just gate-diff` on the tree with the scenario: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/672-rusty-screens-in-the-rail-header.sh` (`compositor sway`), on the debug build,
Rusty's stand-in over a scratch vault, the copy at Chad's 24px UI font and the rail at its default
260px. `lines_meet` read the border rows of every view: 42 and 90 in the rail and in the main
column each time.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001, 003, 005 | `672-01-projects` | Projects (on), Brain, Today, Graph, Tasks, Decisions, `…`, then the folder `+`; the filter row's line on the tab bar's |
| REQ-005 | `672-02-filtering` | `zz` and the clear button; No matches; the same lines |
| REQ-001, 004, 005 | `672-03-brain` | Brain on, the same screens, New Page `+`; no row of screen buttons; the search row's line on the tab bar's; the scratch vault's `notes` folder |
| REQ-002 | `672-04-graph` | after Projects then the header's Graph: the Graph tab in front, the stand-in's one node; the stand-in logged `brain_graph` |
| REQ-003 | `672-05-overflow` | `…` open: Memory, Skills, Secrets with their icons |
| REQ-003 | `672-06-wide` | the rail dragged to 400px: all seven screens and the folder `+`, no `…` |

The exploratory run before the gate missed `…` (a guessed x) and found the 1px offset; both were
fixed before the gate. Focus: headless sway; Hyprland's one Marley window (Chad's) before and
after, no rule added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed); the in-app guide's Brain view article (before the
  gate); `marley_workbench.md` (the header's screens); `rusty-in-marley.md` (R-D9's row and R4).
  No Zed path touched.
- **Knowledge:** `L-claude-672-the-window-draws-a-line-under-the-title-bar-001`,
  `AD-claude-672-rustys-screens-sit-in-the-rail-header-and-overflow-under-an-ellipsis-001`.
- **Brain:** `decisions/rustys-screens-sit-in-marleys-rail-header-and-overflow-under-an-ellipsis`.
- **Ticket:** closed; the pair archived.
