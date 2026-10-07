---
status: intake
created: 2026-10-06
ticket: <unassigned>
pipeline_spec: <unassigned>
---

# The rail and the center tabs: one list or two, and where things open

Research for a talk with Chad on 2026-10-07. Nothing here is decided.

## What Chad said (2026-10-06)

> So right now here is what is going on. The left pane groups things in panes. However some
> things opened in the main pane dont show up on the left pane. So really now we have two things
> that arent in sync. So for example if i open a file to edit, a knowledge page it doesnt show up
> but only in the middle. But if i open a terminal and a browser it does.
>
> Warp handles this by saying you either need to use the top tabs or the left tabs i believe and
> they split pane it.
>
> The question is do we keep the main pane with tabs or remove them entirely. if we keep the main
> tabs which i actually like then everything open there needs to be opened in the left pane. And
> things need to fall to a default pane like discussed

And earlier the same day: "Maybe we should have a pane thats always open (not closable) that is
the default pane something opens if no pane is obvious? This would include things like the
brain. Maybe instead of the idea on 3 of the group called Rusty we have this instead?" The "group
called Rusty" was a place for everything Rusty's icons open, with a placeholder icon until he has
one.

## What Marley does today

**The rail lists five kinds of thing per project:** center terminals and Browser tabs (read with
`items_of_type::<TerminalView>` and `::<BrowserView>`, `rail.rs:6119-6123`), Zed agent threads
(from the Agent Panel in the right dock), harness sessions, and the container ports the project
holds.

**The center holds every kind of tab**, and most have no row: file editors, Rusty's Page, Graph,
Tasks, Decisions, Memory, Skills and Secrets tabs, the fleet's Agent tab, System One calls, and
Zed's own (project search, diffs, settings, Markdown preview, images). Marley's own center items
are eleven `impl Item` types in `marley_workbench`. Two of them get rows.

**The highlight follows only those two.** `active_rows` (`rail.rs:8209`) marks a row only when
the active center item is a terminal or a Browser tab. With a file or a Rusty page in front, no
row is highlighted.

**Everything opens in the shown project's active pane.** Rusty's tabs, pages, the Agent tab and
System One calls all call `add_item_to_active_pane`. The exceptions are the Browser's
`place_tab`, which splits right when the active pane is busy (`browser.rs:4173-4250`), the
Playwright scripts and the launch splits. So a Rusty page opened while project A is shown lives
in A's workspace, and it disappears from view when the rail switches to B.

## How the others do it

**Warp: one list.** Vertical tabs replace the tab bar: "The vertical tabs panel replaces the
horizontal tab bar with a sidebar" and "The horizontal tab bar is hidden while vertical tabs are
active" (docs.warp.dev/terminal/windows/vertical-tabs/). A row is a pane by default ("each split
pane gets its own row") or a tab ("View as: Tabs"). The settings are "Use vertical tab layout",
"View as", "Tab item", "Density", "Pane title as" and "Show details on hover". There is nothing
to keep in sync because there is one list. Our own observation is
`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`.

**Orca: two lists with different jobs** (`docs/orca_architecture/05-terminal-and-workspace.md`
§2.7, `01-agents-and-sessions.md`). The left sidebar lists worktrees as cards with their agents
("3 working, 1 waiting"), never every tab. Each worktree has its own top tab bar holding every
kind (terminal, editor, diff, browser, agent session) with splits, colours, pins and its own
most-recent stack. A floating workspace outside every worktree holds its own terminal, browser and
Markdown-note tabs. That is Orca's answer to "where does a thing with no project go".

**Zed: top tabs only.** Its Threads Sidebar lists agent threads, never tabs.

## The options for the tabs

**A. Keep the top tabs, and give every center tab a row** (Chad's lean). Rows for files, Rusty's
tabs, the Agent tab and System One calls too, so the rail and the center list the same things,
and exactly one row is highlighted whatever is in front.
- **Work.** Read every center item (`Workspace::items`), not two types, and follow the panes'
  add, remove and activate events. All of it is in Marley's crate, with no Zed hunk.
- **What needs deciding.** A project with forty open files would make a long rail. Files could
  fold under one "Files (40)" row per project, as Containers fold today. Zed's preview tab, the
  one that a single click replaces, needs a row that changes with it.
- **Size:** M.

**B. Remove the top tabs, and the rail is the only list** (Warp's way).
`Pane::set_should_display_tab_bar` (`workspace/src/pane.rs:831`) hides a pane's tab bar from
our crate, with no Zed hunk.
- **What goes with the tab bar:** the back and forward arrows (they live in it,
  `pane.rs:3490`), the split and zoom buttons, dragging a tab onto a pane's edge to split, the
  tab menu (close others, pin, copy path), the dirty dot on a changed file, and the preview tab's
  italic.
- **What the rail would need instead:** each of those, plus every row from option A, because the
  rail becomes the only way to reach a tab.
- **Size:** L.

**C. Two lists on purpose** (Orca's way). The rail lists projects and their long-lived things
(terminals, agents, Browser tabs), and the top tabs hold everything. This is today's state named
as the design. It leaves the mismatch Chad does not like.

## The default pane

**What it is:** a place that is always there, that cannot be closed, and that takes whatever
opens without an obvious home.

**Zed's public seams cover it with no Zed hunk.**
- `Pane::set_close_pane_if_empty(false)` (`pane.rs:856`) keeps a pane on screen with no tabs.
  Nothing in Zed calls it today, so the first caller checks how the workspace treats an empty
  center pane.
- `Workspace::add_item(pane, ...)` puts an item in a given pane, not the active one.
- `set_can_split` and `set_render_tab_bar` give the pane its own rules and look.

**What counts as obvious needs a rule.** A proposal: a thing that belongs to a project (a file, a
terminal in its folder, a Browser tab on its port) opens in that project's last active pane. A
thing that belongs to no project (Rusty's screens and pages, the fleet's Agent tab, System One
calls) opens in the default place.

**It can be per project or window-wide.** A pane belongs to one project's workspace, so these are
the two shapes:
- **A fixed pane in each project.** Every project gets one. A Rusty page opened in project A is
  in A's fixed pane, and it is gone from view on a switch to B.
- **One window-wide place.** Marley already has folderless groups (#600, `groups.rs`): named
  groups in the rail with no folder, each a workspace of its own, the window's Home group among
  them. A group called Rusty, created while Rusty is on, listed in the rail with a placeholder
  icon and never closed, would be that place. Every Rusty screen opens there whatever project is
  shown, and with option A its tabs are its rows. This joins Chad's item 3 (the Rusty group) and
  item 5 (the always-open default pane). Orca's floating workspace is the same idea.

## Questions for the talk, one at a time

1. Keep the top tabs and give every tab a row (A), or remove them (B)? Recommendation: A. It
   keeps Zed's tab features, which B would have to rebuild in the rail, and Chad likes the tabs.
2. With A, does each open file get its own row, or do a project's files fold under one row?
3. Where do things with no project open: a Rusty group in the rail that is there for the whole
   window, or a fixed pane in each project? Recommendation: the Rusty group. It survives a
   project switch, and #600 already built the folderless group it needs.
4. Does that place also take the Agent tab and System One calls, or only Rusty's screens?

## Related tickets

#672 moves Rusty's icons into the rail's header and leaves the group to this talk. #671 and #673
do not depend on it.

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via
`/pipeline:plan` when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`.
