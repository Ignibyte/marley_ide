---
pipeline_id: 7c274433-6617-49df-8d6c-6f58eb8d8f2b
ticket: docs/planning/tickets/open/TICKET-699-the-rusty-groups-plus-menu-and-home-page.md
status: Phase 4 — Complete PASS
title: The Rusty group's + menu and its home page
type: feature
slice: Rusty in Marley (docs/marley/rusty-in-marley.md), on Chad's 2026-10-09 ask from daily use
references:
  - docs/planning/pipeline/completed/675-the-rusty-group.spec.md
  - docs/planning/pipeline/completed/679-the-rusty-home-page.spec.md
---

## Title
Give the Rusty group a + menu of Rusty's own quick links, and never show Zed's Welcome page in it:
an empty Rusty group shows its home page.

Chad, 2026-10-09: "Rusty is treated as a panel. in its + i think we should have quick links inside
of there for rust things" and "When rusty panel is empty and i click on it, it defaults to the zed
open project panel. lets default it to basically the home page. or have the home page open all the
time".

## Scope
### In
- The + on the tab bar of a Rusty group's pane opens a menu that starts with Rusty's quick links:
  - **Home**, Rusty's home page;
  - the eight screens, in the home page's order, with their icons: Brain, Today's Note, Graph,
    Tasks, Decisions, Memory, Skills, Secrets;
  - **Open Page…** and the three captures: **Capture to Today…**, **Capture to Inbox…**,
    **Capture a URL…**.

  Zed's own entries follow under a separator. Every other pane's + menu stays Zed's.
- The Rusty group never shows Zed's Welcome page. Whenever it would show no tab, its home page
  opens:
  - when its last tab closes;
  - when it is shown with no tab, from the rail, a switch or a restart that restores it.

### Out (explicitly deferred)
- The home page across a restart (`SerializableItem`): the group shown after a restart gets a new
  one, which reads Rusty again anyway.
- Keeping the home page open while other tabs are open: closing it beside other tabs closes it,
  and it comes back the next time the group would be empty or anything opens there (#679's
  `ensure`).
- Rusty's links in a project's or another group's + menu.
- The Zed layout, which has no Rusty group.

## Reference (§20)
Upstream Zed, workspace: the pane's tab bar + menu (`default_render_tab_bar_buttons` in
`workspace/src/pane.rs`) and its empty state, which draws `welcome::WelcomePage` in a pane with no
item when the project has no worktree. Marley keeps both for every pane but the Rusty group's: it
adds entries ahead of Zed's in that menu, and fills the empty state with Rusty's home page before
Zed's Welcome page shows. No Warp analog: Warp has no Rusty.

### Prior art
- **The code we ship:**
  - `Pane::set_render_tab_bar_buttons` replaces a pane's whole button row; Zed's Terminal Panel
    uses it (`terminal_panel.rs`, `apply_tab_bar_buttons`). Using it here would copy Zed's split
    and zoom buttons into Marley and would have to be put on every pane the group grows. Zed has
    no hook for the + menu's entries alone, so a small Marley hook in `pane.rs` (a global asked for
    leading entries, as `MarleyCloseGuard` is asked before a close) is the smaller diff.
  - `Pane::should_display_welcome_page` and `welcome_page` decide the empty state; filling the
    group with its home page leaves them alone.
  - `workspace::Event::ItemRemoved` (a pane's item left) and
    `MultiWorkspaceEvent::ActiveWorkspaceChanged` (another workspace shown) are the moments to
    look; `cx.subscribe_in(&cx.entity(), window, …)` is how Zed's `call` crate hears its own
    entity with a window.
  - `ui::ContextMenu::{item, action, separator}` and `ContextMenuEntry::{icon, handler}`.
- **Marley:** `home_tab::ensure` (#679) adds the page first in a workspace with none;
  `brain::Screen::ALL` and `brain::open_screen` drive the home page's screen buttons;
  `groups::{Groups, rusty_group, is_rusty}` know the Rusty group (#675); the Rusty actions
  `OpenPage`, `CaptureToToday`, `CaptureToInbox` and `CaptureUrl` open their forms.
- **Behavior maps:** `docs/zed_architecture/` has nothing on the + menu or the empty pane.

## UI proof
`script/e2e/699-the-rusty-groups-plus-menu-and-home-page.sh`, under `compositor sway`, with
`marley_rusty`'s stand-in `rusty-mcp` over a scratch state folder, never the user's Rusty (R-D8).

Shots:
- `699-01-plus-menu`: the Rusty group's + menu open, Rusty's links above Zed's entries.
- `699-02-from-menu`: Tasks chosen from it: the Tasks tab in the Rusty group.
- `699-03-closed-all`: every tab of the Rusty group closed: the home page shows, not Zed's
  Welcome page.
- `699-04-shown-empty`: the group restored with no tab after a restart, shown from the rail: the
  home page.
- `699-05-project-plus`: a project's + menu, unchanged.

## Locked-In Decisions
- **D1:** the + menu's leading entries come from a Marley hook in Zed's `pane.rs`: a global
  `MarleyNewItemMenu`, asked with the pane's workspace and the menu, and returning it with entries
  added. Without the global, or for any workspace but a Rusty group's, the menu is Zed's.
- **D2:** Rusty's links come first and Zed's entries stay below a separator, so nothing the menu
  did is lost.
- **D3:** the rule is "the Rusty group never shows no tab": it is checked when an item leaves the
  group's workspace, when the window shows another workspace, and when the groups change (which
  covers a restored group's adoption). It acts only while Rusty is on.
- **D4:** the screens open through `brain::open_screen` and the home page through
  `home_tab::open_later`, as the home page's buttons do; the page picker and the captures through
  their actions, so their forms open as from the palette.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens the + menu of a pane in the Rusty group, the system shall list Home, the eight screens, Open Page… and the three captures above Zed's own entries. | Shot 699-01 |
| REQ-002 | WHEN the user chooses a screen from that menu, the system shall open that screen's tab in the Rusty group. | Shot 699-02 |
| REQ-003 | WHEN the last tab of the Rusty group closes while Rusty is on, the system shall show the Rusty home page in the group, not Zed's Welcome page. | Shot 699-03 |
| REQ-004 | WHEN the Rusty group is shown with no tab, the system shall open its home page there. | Shot 699-04 |
| REQ-005 | WHILE a pane belongs to a project or a group other than Rusty's, its + menu shall list Zed's entries only. | Shot 699-05 |
| REQ-006 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the `pane.rs` hook and its ledger row; the menu and the empty-group rule in
  `rusty/home_tab.rs`; `groups::is_rusty_workspace`; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
