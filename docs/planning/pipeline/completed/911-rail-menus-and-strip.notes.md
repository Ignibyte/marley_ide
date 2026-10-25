# The rail's menus and strip — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-911-rail-menus-and-strip.md
- **Pipeline spec:** 911-rail-menus-and-strip.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 3, "the menus' missing entries"; B-rail-chrome.md, RailMenus
  and RailStates).
- **Classification / tier:** feature, M. Marley crate only (`marley_workbench`). Last in this
  batch's build order (906, 905, 907, 908, 909, 910, 911); it shares `rail.rs` with #910, which
  changes the rows and headers, not the menus.
- **Recall (§18.3):**
  - PR-claude-785-a-menu-that-dispatches-a-pane-action-names-the-pane-as-its-context-001: a menu
    entry that dispatches a pane action needs the pane as its context. Split Right calls
    `Pane::split` directly on the terminal's pane instead, so no dispatch is involved.
  - PR-claude-615-a-menus-trigger-owns-its-tooltip-001 and
    L-claude-758-a-rail-rows-tooltip-covers-its-right-click-menu-in-a-shot-001: the row's tooltip
    stays inside the menu's trigger; move the pointer off before a shot.
  - L-claude-656-an-inline-editor-takes-enter-and-escape-from-a-menu-context-001 and
    L-claude-726-enter-in-a-single-line-editor-is-menu-confirm-001: the new-group field wraps its
    editor in `key_context("menu")` with `menu::Confirm` and `menu::Cancel` handlers.
  - L-claude-713-the-empty-space-menu-selects-no-entry-001,
    L-claude-832-a-right-click-menu-opens-at-the-pointer-so-click-its-entries-inside-001 and
    L-claude-822-a-menu-entry-a-scenario-counts-to-moves-when-a-ticket-adds-one-001: the scenario
    clicks entries by point, about 40 px right of the right-click, read from a probe run's shot.
    Adding Fold and three row entries moves points in older scenarios (#600, #613, #713, #735):
    note it for the regression pass.
  - L-claude-749-a-new-group-row-moves-the-rail-rows-a-scenario-clicks-001: making a group moves
    the rows below; take the group's shots last, or fold first.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: the stand-in `claude` goes first on
    the terminals' PATH through the scenario's `.bashrc`, never the box's real CLI.
  - AD-claude-555-a-block-goes-to-an-agent-by-reference-or-inline-redacted-001: Send Block to
    Agent reuses `send_block::send`, so the redactor and the target picker come with it.
  - AD-claude-600-projectless-groups-are-marley-records-over-folderless-workspaces-001: a new
    group is `groups::make`, whatever asks for its name.
- **Discovery:**
  - `crates/marley_workbench/src/rail.rs`: `HeaderMenu` (506), `project_context_menu`
    (5970-6074), `group_context_menu` (5929-5965), `fold` (3593), `deploy_empty_menu`
    (4717-4790), `new_group` (4611), `terminal_context_menu` (6761-6835), `toggle_turns`
    (4815), `render_turns` (7033), `activate_terminal` (3076), `render_add_project` (5709). The
    callers that build `HeaderMenu` and call `terminal_context_menu` pass the new fields.
  - `crates/marley_workbench/src/rail_strip.rs`: `render_strip` (259-320).
  - `crates/marley_workbench/src/groups.rs`: `DEFAULT` (33), `GroupNamePrompt::confirm`
    (744-762), `make`.
  - Read only: `crates/ui/src/components/context_menu.rs` (`custom_entry` 643, its row 1453,
    label colours 1818), `crates/workspace/src/pane.rs` (`split` 2588, `SplitMode` 216),
    `crates/terminal_view/src/terminal_view.rs` (`clone_on_split` 2355),
    `crates/marley_workbench/src/send_block.rs`, `blocks.rs` (222-234).
- **Decisions:** D1–D8 in the spec. Fold is added to the group menus too, because the board's
  Default and named groups fold like projects.

- **Promoted 2026-10-25:** seams re-read after #910: `project_context_menu`, `group_context_menu`,
  `terminal_context_menu` (its `moves` tuple), `deploy_empty_menu`, `HeaderMenu`, `Rail::fold`,
  `toggle_turns`, `new_group` (the modal `groups::prompt`), `render_strip`, `render_add_project`,
  `groups::make`, `Pane::split` with `SplitMode::ClonePane`, `ContextMenu::custom_entry`;
  `groups::DEFAULT` is `pub(crate)` since #910.

### Design
- A free function `danger_entry(menu, label, handler) -> ContextMenu` in `rail.rs` adds a
  `custom_entry` whose render is `Label::new(label).color(Color::Error)`. Close Project and the
  row's Close use it.
- `HeaderMenu` gains `index: usize` and `fold: HeaderFold` (an enum `Open | Folded`, not a bool),
  set where the header's menu is built. `project_context_menu` and `group_context_menu` add
  `Fold`/`Unfold` before the closing entry, calling `rail.fold(index, …)`.
- `terminal_context_menu` takes a `TerminalMenu { id: u64, moves: Vec<MoveTarget>, turns: usize }`
  in place of the `(u64, &[(usize, String)])` tuple (Google style: no multi-field tuples in
  signatures); `MoveTarget` names the place and the name. The new entries go between Review Last
  Reply and Move to Project. Send Block to Agent picks
  `MarleyBlockSelection::selected(&terminal, cx)` or the last live block's index, and is
  `disabled` when there is none. Split Right: `activate_terminal`, then the terminal's pane
  (`workspace.pane_for(&view)`) `split(SplitDirection::Right, SplitMode::ClonePane, …)`.
- `deploy_empty_menu` formats its two labels with `groups::home_name()` (new `pub(crate)` fn
  returning `DEFAULT`).
- The strip's foot: `render_strip` appends a bottom row with an `IconButton` `IconName::Plus`
  inside a `PopoverMenu` over `SidebarRecentProjects::popover`, a new
  `strip_add_project_menu: PopoverMenuHandle<SidebarRecentProjects>` on the rail, and an anchor
  that opens upward and to the right.
- The new-group field: the rail holds `new_group: Option<NewGroupField>` (the editor and its
  focus-out subscription). `new_group` sets it and focuses the editor instead of calling
  `groups::prompt`; the rail's render draws it at the foot of the projects view (border
  `text_accent`, "New group" muted, the editor, "Enter" muted). Confirm calls `groups::make`
  with the trimmed name unless it is empty; Cancel and focus-out clear the field.
  `groups::prompt` stays for Rename Group….
- Files: `rail.rs`, `rail_strip.rs`, `groups.rs`, all Marley crate. No Zed path changes, so no
  row in `docs/marley/zed-touchpoints.md`.

### Visual check plan
- REQ-001: right-click the project header (`911-01-project-menu`), click Fold
  (`911-02-folded`); right-click again to see Unfold is not required, the folded header proves
  the action.
- REQ-002: the same shot shows "Close Project" red and last; the action is unchanged, so the diff
  review confirms it still calls `remove_project`. Clicking it would end the run's project.
- REQ-003: unfold, right-click the stand-in's terminal row after its turn (`911-03-row-menu`); the
  same in Marley Day (`911-12-day`).
- REQ-004: click Turns (1) (`911-04-turns`).
- REQ-005: right-click the shell's row, click Send Block to Agent; with one agent terminal the
  send needs no picker; the stand-in's screen shows the block (`911-05-sent`).
- REQ-006: right-click the shell's row, click Split Right (`911-06-split`).
- REQ-007: right-click empty space under the rows (`911-07-empty-menu`).
- REQ-008: click New Group…, type `Errands` (`911-08-group-field`), Enter (`911-09-group-made`).
  Last among the rail shots, since the new group moves rows.
- REQ-009: click Shrink, shot the strip (`911-10-strip-plus`), click the `+`
  (`911-11-strip-popover`).

### Risks
- A `custom_entry` row draws no key binding and no icon slot; Close and Close Project have
  neither, so nothing is lost. If its label sits at another inset than the entries, wrap it in
  the same padding the shot shows.
- A popover at the strip's foot may open off the window's bottom with the wrong anchor; the shot
  checks it.
- The inline field takes focus from the rail; Escape and focus-out must both clear it, or a
  half-typed name stays at the foot.
- Turns need #566's stand-in with hook events, which #509's scenario already runs; borrow its
  set-up rather than write a new one.

## Phase 2 — Code
- **Built:** as scoped, all in `rail.rs` and `rail_strip.rs`; no Zed path.
- **Deviations:** `saved_rail_state` came out of `Rail::new` and `send_block_entry` out of the
  terminal menu, to keep both under clippy's length; the new-group field's placeholder reads
  "Group name" under the "New group" label.
- **Review:** the red entries sit in Zed's own list; Split Right and New Group's make defer or
  run outside the rail's update where they touch panes and windows. Google style:
  `TerminalMenu` replaces a tuple.
- **Gate:** `just gate-diff` 17/17 green after `too_many_lines` (twice), `redundant_clone` and
  `needless_pass_by_ref_mut`.

## Phase 3 — Test
- **Scenario:** `script/e2e/911-rail-menus-and-strip.sh` (sway): `a` with a shell that ran `echo
  hello` and a stand-in Claude Code whose one turn writes notes.md (so the rail lists it) and that
  echoes what it is sent. Coordinates from a probe run of each menu.
- `911-01-project-menu`: Fold, then Close Project in red, last (REQ-001, REQ-002).
- `911-02-folded`: `a` folded, reading "quiet" (REQ-001).
- `911-03-row-menu`: Rename, Open Agent Here…, Review Last Reply, Send Block to Agent, Split
  Right, Move to Project, Close in red (the shell has no turns) (REQ-003).
- `911-04-turns`: Turns (1) on the agent's row: "Tidy the imports · 1 file" listed (REQ-004).
- `911-05-sent`: the stand-in printed the block: `$ echo hello`, `hello`, `exit 0` (REQ-005).
- `911-06-split`: a new shell pane at the right (REQ-006).
- `911-07-empty-menu`: "New Terminal in Default", "New Browser Tab in Default" (REQ-007).
- `911-08-group-field`: "New group", the field, "Enter", on an accent border (REQ-008).
- `911-09-group-made`: the Docs group under `a`, the field gone (REQ-008).
- `911-10-strip-plus`: the strip's D, A, D tiles and the `+` at its foot (REQ-009).
- `911-11-strip-popover`: Add Project's popover open upward from the `+` (REQ-009).
- `911-12-day`: the row menu in Marley Day, Close in red (REQ-003).

## Phase 4 — Complete
- CHANGELOG, `docs/marley_architecture/marley_workbench.md`. No Zed path.
- Knowledge: nothing new; the run found no bug.
