---
pipeline_id: f168244f-96fb-48eb-a112-73e55d2930d1
ticket: docs/planning/tickets/open/TICKET-911-rail-menus-and-strip.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The rail's menus and strip as the RailMenus and RailStates boards draw them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/B-rail-chrome.md]
---

## Title
The rail's right-click menus gain the RailMenus board's entries and words: Fold and a red Close
Project on a header, Turns (N), Send Block to Agent, Split Right and a red Close on a terminal
row, "New Terminal in Default" in the empty space. The shrunk strip gets a `+` at its foot, and
New Group… names the group in a field at the rail's foot, as RailStates and RailMenus draw them.

## Scope
### In
- **A red closing entry** (`marley_workbench/src/rail.rs`, Marley crate): a small helper that
  adds a menu row whose label is `Color::Error`, through Zed's public
  `ContextMenu::custom_entry`. The project menu's Close Project and the terminal row's Close use
  it. No Zed hunk.
- **Project header menu** (`project_context_menu`, Marley crate):
  - "Fold" while the project is open, "Unfold" while it is folded, before the closing entry. It
    calls `Rail::fold` with the header's index. `HeaderMenu` gains the index and whether the
    project is open.
  - "Remove Project" reads "Close Project", in red, last. Its action stays `remove_project`.
- **Group header menus** (`group_context_menu`, Marley crate): the same Fold / Unfold entry, for
  a named group and for Default.
- **Terminal row menu** (`terminal_context_menu`, Marley crate), in the board's order: Rename,
  Open Agent Here…, Review Last Reply, Turns (N), Send Block to Agent, Split Right, Move to
  Project ▸, then a red Close.
  - "Turns (N)" shows only when the terminal has turns. It opens or closes the row's turns list,
    as the card's disclosure does (`toggle_turns`).
  - "Send Block to Agent" sends the terminal's selected block, else its newest block, through
    `send_block::send` with `Request::Bare`. It is disabled while the terminal has no block.
  - "Split Right" brings the terminal to the front, then splits its pane to the right with
    `Pane::split(SplitDirection::Right, SplitMode::ClonePane, …)`: a new shell beside it, as
    Zed's pane split does for a terminal.
  - The function's `moves: (u64, &[(usize, String)])` tuple becomes a `TerminalMenu` struct
    with named fields (the terminal's id, its move targets, its turn count), as `HeaderMenu`
    is for a header.
- **Empty-space menu** (`deploy_empty_menu`, Marley crate): "New Terminal in Default" and "New
  Browser Tab in Default", with the home group's name read from `groups.rs` (a `pub(crate)`
  accessor for its `DEFAULT` name).
- **Strip foot** (`rail_strip.rs`, `render_strip`, Marley crate): a `+` icon button at the
  strip's foot, under the tiles, tooltip "Add Project". It opens the same recent-projects
  popover as the rail header's Add Project (`SidebarRecentProjects::popover`), with a handle of
  its own, anchored to open upward from the foot.
- **New group inline** (`rail.rs` and `groups.rs`, Marley crate): New Group… shows a field at
  the rail's foot: "New group", a one-line editor, and an "Enter" hint, with an accent border.
  Enter makes the group through `groups::make` (as `GroupNamePrompt::confirm` does); Escape or
  losing the focus closes the field. An empty name makes nothing.

### Out (explicitly deferred)
- "Link to a Brain Page…" in the project header menu: it needs a brain-page picker and a place
  to keep the link (the review's size M). A follow-up ticket.
- Rename Group… stays the modal prompt. Only New Group… goes inline.
- The project menu's own entries the board does not draw stay: Move Project Up and Down, the logo
  choices (#831), Archived Threads (#616), Send Failed Checks to Agent… (#718), Clear Browser
  Data… (#581). The empty-space menu keeps Reopen Thread…, and the row menu keeps Move to
  Project ▸ (#613).
- The strip's flyout and its search popover as RailStates draws them (a compact flyout; a match
  count, where-lines and a key-hint footer), and the rail's bordered "Filter rows and projects"
  field: looks, not in this ticket's list. A follow-up ticket.
- The Threads view rows' agent tiles and times (RailMenus): a follow-up ticket.
- The rail rows' second lines and the headers' summaries: #910.
- Glyph tiles in place of the rail's icons: deferred by #827 ("the icons stay").

## Reference (§20)
Upstream Zed (`ui::ContextMenu` and `workspace::Pane`): the menus stay Zed's context menus, a red
entry is a `custom_entry` row drawn in the same list item and selected by the same keys; Split
Right is Zed's pane split with `SplitMode::ClonePane`, which for a terminal starts a new shell
(`TerminalView::clone_on_split`). The new-group field follows Zed's project panel rename: a
one-line editor in place, Enter as `menu::Confirm`, Escape as `menu::Cancel`. Everything else is
N/A — Marley-specific: the rail and its menus are Marley's (#600, #613, #820, #830).

### Prior art
- `Rail::project_context_menu` (`crates/marley_workbench/src/rail.rs:5970-6074`): the `+`
  entries, Move Project Up/Down, the logo choices, Archived Threads, Send Failed Checks to
  Agent…, Clear Browser Data…, Remove Project. No Fold, no red entry.
- `HeaderMenu` (`rail.rs:506-521`): carries the key, the workspace, the ends, closed, archived,
  failed checks; no index, no expanded state.
- `Rail::group_context_menu` (`rail.rs:5929-5965`): Default's menu is the `+` entries and a
  label; a named group's adds Rename Group… and Remove Group.
- `Rail::fold` (`rail.rs:3593-3615`): folds or unfolds the header at an index unless a filter
  decides what shows, for a project or a group (`groups::toggle_expanded`). The Fold entry calls
  it.
- `Rail::terminal_context_menu` (`rail.rs:6761-6835`): Rename, Open Agent Here…, Review Last
  Reply, Move to Project ▸, Close, with `moves` as a tuple.
- `Rail::render_turns` (`rail.rs:7033-7065`) and `toggle_turns` (`rail.rs:4815`): the "Turns
  (N)" disclosure under a terminal's card.
- `send_block::send` (`send_block.rs`) and `blocks::send_focused` (`blocks.rs:222-234`): a
  block to an agent, the selected block by `MarleyBlockSelection::selected`.
- `Pane::split` (`crates/workspace/src/pane.rs:2588`), `SplitMode` (`pane.rs:216-224`) and
  `TerminalView::clone_on_split` (`crates/terminal_view/src/terminal_view.rs:2355`): a
  terminal's split clones a new shell in the project's directory. `Rail::activate_terminal`
  (`rail.rs:3076`) brings a terminal to the front first.
- `ContextMenu::custom_entry` (`crates/ui/src/components/context_menu.rs:643`, drawn at
  `:1453` in the same inset `ListItem` as an entry, selectable). `ContextMenuEntry`'s label
  colour is only Default or Disabled (`context_menu.rs:1818-1822`), so a red entry needs
  `custom_entry` or a Zed hunk; `custom_entry` needs none.
- `Rail::deploy_empty_menu` (`rail.rs:4717-4790`): New Group…, New Terminal, New Browser Tab,
  New Agent…, Reopen Thread…, each made in the home group by `in_home`. `groups.rs:33`'s
  `DEFAULT` is the home group's name.
- `render_strip` (`rail_strip.rs:259-320`): Open, search, the tiles; no foot. The rail header's
  Add Project (`render_add_project`, `rail.rs:5709-5737`) is a `PopoverMenu` over
  `SidebarRecentProjects::popover` with `add_project_menu` as its handle.
- `groups::prompt` and `GroupNamePrompt` (`groups.rs:697-790`): the modal; `confirm` calls
  `groups::make(…, GroupKind::Named, …)`.
- The boards: RailMenus' `projectMenu`, `rowMenu` and `emptyMenu` (`danger` ink `#ff8f80`, the
  theme's error red) and its "New group" field; RailStates' strip foot `+`.
- `docs/zed_architecture/`, `docs/warp_architecture/`: nothing on rail menus. Searched the
  workbench for "Brain Page" and "link.*brain": no picker to reuse, hence Out.

## UI proof
Scenario `script/e2e/911-rail-menus-and-strip.sh` (sway): a scratch repository opened as the
project, a shell with one finished command (two blocks), and #509's stand-in `claude` (#566's,
on the terminals' PATH through the scenario's `.bashrc`) run in a second terminal for one turn
that writes a file, so its row has "Turns (1)".
- `911-01-project-menu`: the project header's menu open: the `+` entries, Fold, and a red Close
  Project last (REQ-001, REQ-002).
- `911-02-folded`: Fold clicked: the project folded (REQ-001).
- `911-03-row-menu`: the stand-in's terminal row's menu: Rename, Open Agent Here…, Review Last
  Reply, Turns (1), Send Block to Agent, Split Right, Move to Project ▸ (when shown), a red Close
  (REQ-003).
- `911-04-turns`: Turns (1) clicked: the turns list open under the card (REQ-004).
- `911-05-sent`: on the shell's row, Send Block to Agent clicked: the stand-in's prompt holds the
  block (REQ-005).
- `911-06-split`: on the shell's row, Split Right clicked: two panes, the new shell at the right
  (REQ-006).
- `911-07-empty-menu`: the empty space's menu: "New Terminal in Default", "New Browser Tab in
  Default" (REQ-007).
- `911-08-group-field`: New Group… clicked, a name typed: the field at the rail's foot (REQ-008).
- `911-09-group-made`: Enter: the new group's header in the rail, the field gone (REQ-008).
- `911-10-strip-plus`: the rail shrunk: the `+` at the strip's foot (REQ-009).
- `911-11-strip-popover`: the `+` clicked: the recent-projects popover (REQ-009).
- `911-12-day`: the theme set to Marley Day, the terminal row's menu open (REQ-003).

## Locked-In Decisions
- D1 — The board's entries are added; every entry Marley has today stays (the logo choices,
  Archived Threads, Clear Browser Data…, Move to Project ▸, Reopen Thread…). The board shows a
  menu's shape, not a list to cut to.
- D2 — "Close Project" is Remove Project renamed: it removes the project from the window, as
  now (Zed's own intent is `RemovalIntent::CloseProject`). It is red and last.
- D3 — A red entry is a `ContextMenu::custom_entry` with a `Color::Error` label, drawn in the
  same list item as the other entries, so no Zed hunk.
- D4 — Fold reads "Unfold" on a folded header, and does nothing while a filter decides what
  shows (as `Rail::fold`).
- D5 — Send Block to Agent sends the selected block, else the newest one, as Send to Agent in
  the Block menu sends a block.
- D6 — Split Right clones the terminal into a new shell at the right (`SplitMode::ClonePane`),
  as Zed's split menu does for a terminal; it does not move the row's terminal.
- D7 — The strip's `+` opens the same popover as the rail header's Add Project.
- D8 — Only New Group… goes inline; Rename Group… keeps the modal.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project or group header is right-clicked, its menu shall list Fold (Unfold while folded), and choosing it shall fold (or unfold) that header. | `911-01-project-menu`, `911-02-folded` |
| REQ-002 | A project header's menu shall end with "Close Project" in the error red, which removes the project as Remove Project did. | `911-01-project-menu`, the diff review |
| REQ-003 | WHEN a terminal row is right-clicked, its menu shall list Rename, Open Agent Here…, Review Last Reply, Turns (N) when it has turns, Send Block to Agent, Split Right, Move to Project when another project is open, and a red Close, in that order, in Marley Night and Marley Day. | `911-03-row-menu`, `911-12-day` |
| REQ-004 | WHEN Turns (N) is chosen, the row's turns list shall open (or close). | `911-04-turns` |
| REQ-005 | WHEN Send Block to Agent is chosen, the terminal's selected or newest block shall go to an agent as Send to Agent sends it. | `911-05-sent` |
| REQ-006 | WHEN Split Right is chosen, the terminal's pane shall split with a new shell at the right. | `911-06-split` |
| REQ-007 | The empty space's menu shall read "New Terminal in <home group>" and "New Browser Tab in <home group>". | `911-07-empty-menu` |
| REQ-008 | WHEN New Group… is chosen, a "New group" field with an Enter hint shall show at the rail's foot, and Enter shall make the group and close the field. | `911-08-group-field`, `911-09-group-made` |
| REQ-009 | WHILE the rail is a strip, a `+` shall sit at its foot, and a click shall open the Add Project popover. | `911-10-strip-plus`, `911-11-strip-popover` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the menus, the red entry helper, the strip's `+` and the inline field in
  `rail.rs`, `rail_strip.rs` and `groups.rs`; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
