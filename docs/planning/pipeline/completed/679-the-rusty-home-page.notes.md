# The Rusty home page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-679-the-rusty-home-page.md
- **Pipeline spec:** 679-the-rusty-home-page.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-07 (quoted in the ticket).
- **Classification / tier:** feature; `marley_workbench` only.
- **Pre-flight:** green; #678 committed and installed; cargo idle.
- **Recall (§18.3):** #655's project view reads (due, groups, tasks per group); #654's recents;
  #675/#678 (`in_rusty_group`, the Brain tab's `show_page`).
- **Discovery:** `knowledge_panel.rs` `load_project` and `open_tasks`; `page_picker.rs` `Recent`;
  `marley_rusty::{decisions::{BRAIN_DUE, due_from_answer}, tasks::{LIST_TASK_GROUPS, LIST_TASKS,
  groups_from_answer, tasks_from_answer}, vault::name_of}`; `brain::{Screen, open_screen}`; the
  rail's `render_screens`, `screens_that_fit`, `render_more_screens`, `open_screen`,
  `RUSTY_GROUP_ICON`.

### Design
- **`rusty.rs`**: `RUSTY_ICON` (Blocks, the placeholder); `OpenHome` (`rusty: open home`);
  `in_rusty_group` makes sure of the home tab before its open.
- **`rusty/home_tab.rs`** (new): `RustyHome` (an `Item`, title Rusty, the icon); `open_later`;
  `ensure` (adds it at the first place of the active pane without bringing it forward); its reads
  (due, groups, each group's open tasks) on open, on `Announced` and on connecting; the cards.
- **`rusty/brain_tab.rs`**: `open_page_later(workspace, slug)`, the tab on a page.
- **`rusty/page_picker.rs`**: `recent_slugs`.
- **`rail.rs`**: the header's Rusty button and PROJECTS; the screens' row, its fit and `…` go;
  `RUSTY_GROUP_ICON` becomes `rusty::RUSTY_ICON`.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001 | Rusty's stand-in on, a project | `679-01-header` |
| REQ-002, 003 | clicks Rusty | `679-02-home` |
| REQ-004 | clicks the recent page | `679-03-recent` |
| REQ-005 | clicks Home's row, then Graph | `679-04-screen` |

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall; discovery.
- [x] Mint the pair; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-07)
### Built
- **`rusty/home_tab.rs`** (new): `OpenHome`; `open_later`; `ensure`; `RustyHome` with its reads
  (`load`, `task_rows`) and cards (`render_screens`, `render_recent`, `render_due`,
  `render_tasks`, `card`, `row`, `muted`); `Item`.
- **`rusty.rs`**: `RUSTY_ICON` (Blocks); `pub mod home_tab` and its `init`; the re-exports
  `OpenHome` and `open_home_later`; `in_rusty_group` runs `home_tab::ensure` first.
- **`rusty/brain_tab.rs`**: `open_page_later`.
- **`rusty/page_picker.rs`**: `recent_slugs`.
- **`rail.rs`**: `render_rusty_button` and PROJECTS while connected; `screens_that_fit`,
  `render_screens`, `render_more_screens`, `Rail::open_screen`, the header's `button` argument and
  `RUSTY_GROUP_ICON` gone; the group header takes `rusty::RUSTY_ICON`.
- **The guide**: the Brain article names the home page; each screen's item says where its button is.

### Deviations
- **A double lease, found by the check.** The first run's click on a recent page ended the run's
  Marley: `open_page_later` called `BrainTab::show_page` inside the workspace's update, and
  `show_page` reads that workspace for its language registry. It now defers `show_page`, as the
  tree's opens do (F-claude-679-…).
- **The recent pages card shows the copy's own recents**: the run's profile is a copy of the
  user's, whose key-value store keeps the page picker's recents. The scenario opens its page first so
  it leads; the shots stay in the scratchpad.

### Review
- `ensure` adds the page without focus; the open that follows brings its own tab forward.
- A Rusty button only shows while Rusty is connected; `capture::ready` still guards the opens.

### Gate
`just gate-diff` on the tree with the scenario and the fix: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/679-the-rusty-home-page.sh` (`compositor sway`), on the debug build, Rusty's stand-in over
a scratch vault (a page, a decision due yesterday) and `tasks.json` (Home and Work).

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001 | `679-01-header` | the header: the Blocks button, PROJECTS, the folder +; no row of screens |
| REQ-002, 003 | `679-02-home` | the Rusty group with `Rusty` first and `a-note`; the home tab: RUSTY with eight buttons (Brain to Secrets), RECENT PAGES led by `a-note notes`, FOLLOW-UPS DUE "Ship the home page" with yesterday's date in red, TASKS with the four open tasks and their lists (the completed one left out) |
| REQ-004 | `679-03-recent` | `a-note` clicked: the Brain tab in front on `notes / a-note`, after `Rusty` |
| REQ-005 | `679-04-screen` | the Rusty button, then Graph: the Graph tab beside the home tab, `Rusty` still first |

The run before the fix ended the run's Marley at the recent page's click (F-claude-679-…); after it,
no panic in the log. Focus: headless sway; Hyprland's one Marley window (Chad's) before and after,
no rule added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: the Rusty home page); the in-app guide (before the gate);
  `marley_workbench.md` (a new section); `rusty-in-marley.md` (R4).
- **Knowledge:** `F-claude-679-showing-a-page-inside-the-workspaces-update-001`,
  `AD-claude-679-one-rusty-button-and-a-home-page-in-place-of-the-screens-row-001`.
- **Brain:** `decisions/one-rusty-button-opens-a-rusty-home-page-that-replaces-the-rails-screen-icons`.
- **Ticket:** closed; the pair archived.
