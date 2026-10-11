---
pipeline_id: 7b364324-b369-424b-9da3-6a9de5cb1345
ticket: docs/planning/tickets/closed/TICKET-744-one-new-agent.md
status: Phase 4 — Complete PASS
title: One New Agent
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/743-the-rails-top.spec.md
  - docs/planning/pipeline/completed/734-an-agent-thread-starts-in-a-tab.spec.md
  - docs/planning/pipeline/completed/735-the-new-agent-picker.spec.md
  - docs/planning/pipeline/completed/737-the-threads-page.spec.md
  - docs/planning/pipeline/completed/699-the-rusty-groups-plus-menu-and-home-page.spec.md
---

## Title
One way to start an agent. (Reopen Thread… and threads opening only in tabs left this ticket at
its promotion for #772, the second half of the split its notes proposed.) Today a row's `+` offers New
Agent… (#735's picker), New Agent Thread (a submenu that starts a thread on the row's folder
without asking) and a list of agent CLIs, while the status bar's sparkle opens Zed's Agent Panel,
which draws Open Project and starts nothing in a group with no folder. Chad, 2026-10-11: "just 1
singular open a agent for the left pane and the sparkle. Meaning if its a project then default to
the project folder chosen but allow to switch. Remove New Agent THread with just New Agent. add a
New Agent from Thread or something that restarts", and of that last, "my thought is that you would
re open it".

## Scope
### In
- **New Agent…** is the one entry that starts an agent: the rail row's `+`, every pane's `+` (the
  tab bar's), the status bar's sparkle in the Marley layout, and the palette (`marley: new agent`).
  It opens #735's picker.
- **Where starts on the project.** When the picker opens in a group with a project, Where's first
  row is the project's root folder, marked "this project"; #735's guess (the active terminal's
  folder, else the active file's project) follows when it is another folder. In a group with no
  folder, #735's guess leads as today. From a row's `+`, the group is that row's.
- **The row's `+` loses** New Agent Thread and the Agent CLIs list. It keeps New Terminal, New
  Browser Tab, New Agent in Worktree (a git project) and the launch configs.
- **The pane's `+`** gets New Agent… at its head through Zed's `MarleyNewItemMenu` hook (#699),
  which #743 left with no user; both layouts, while AI is on. #772 adds Reopen Thread… beside it.
- **The sparkle in the Marley layout.** `agent::ToggleFocus` (the sparkle, `ctrl-?`),
  `agent::Toggle`, `agent::FocusAgent`, `agent::NewThread` (`ctrl-n` in a thread) and
  `agent::ToggleNewThreadMenu` open the New Agent picker in the shown group. They are caught
  before Zed's handlers, as `routing.rs` catches the Terminal Panel's toggles. In Zed's layout they
  reach Zed unchanged. (`agent::NewExternalAgentThread` is dispatched only from inside the panel,
  `agent_panel.rs` 6059 and 9368, and its one field is private, so it is left to Zed.)
- `marley: new agent thread` stays an action for keymaps and #749's `marley agent-thread`, and
  leaves the palette.
- The guide (`docs/marley/guide.md`, the in-app page `crates/marley_workbench/guide/index.html`)
  and the walkthrough: New Agent Thread gone, the sparkle.

### Out (explicitly deferred)
- **#772:** Reopen Thread…, and every opener (the Threads page, the rail's thread rows and
  Archived Threads, the inbox's Manager entry, a notification) showing a thread in a tab, never the
  panel; Move Thread to Panel and Open in Center leaving the Marley layout's menus.
- Zed's ways into the right dock that start no thread: a dock restored open, `workspace: toggle
  right dock`, `agent: open settings` (the panel's own settings page), `agent: manage skills`.
  #747 makes the right dock open as tabs, and the Agent Panel is one of its panels; it decides
  where the panel's settings page goes.
- CLI sessions in Reopen Thread…: #754 adds them to the Threads list.
- Home's NEW AGENT card keeps its New Agent… and its per-CLI buttons; Chad named the rail and the
  sparkle.
- New Agent in Worktree folded into the picker's Where: it makes a worktree, a different job.
- The panel's thread kept when Zed's layout is chosen again: a thread moved into a tab stays there.

## Reference (§20)
Upstream Zed, `agent_ui`: the Agent Panel's new-thread entry points (`AgentPanel::toggle_focus`,
`NewThread`, `ToggleNewThreadMenu`) and its thread history (`ThreadsArchiveView`, every
`ThreadMetadataStore` entry, newest first, searched by title then folder). Behavior kept: Zed's
own actions and keys still mean "a new agent thread", Zed's store is the list, Zed's
`ConversationView` draws the thread. Marley changes where they land in its own layout: the picker
and a tab instead of the panel. `workspace`'s pane `+` menu keeps its own entries below Marley's
(#699's hook).

### Prior art
- **The code we ship:**
  - `agents.rs`: `show_picker` (726), `places_for` (751), `guess` (788), `Place` (829),
    `NewAgentDelegate::confirm` (1252) and `Start::run_in` (1041). The first-row rule is a change
    to `places_for`; `thread_tab::default_folder` (`thread_tab.rs:936`) already names the root.
  - `rail.rs` (at 19dda499a2; #743 is moving the lines): `render_project_menu` (5110) builds the
    `+` (New Agent… 5158, New Agent Thread 5167), `agent_menu` (5191), `agent_cli_entries`
    (5221), `worktree_agent_entries` (5253), `new_agent_picker` (3009), `new_agent_thread`
    (3025); thread rows open through `open_thread` (2823) → `open_thread_with` (2866) →
    `load_thread` (371) and `show_thread` (395), which focus the panel; `render_thread_row`
    (5373) has Open in Center (5457); `open_manager_thread` (1971) falls back to
    `agents::start_thread` (`agents.rs:192`), which focuses the panel.
  - `threads_page.rs` `open_thread` (310): the funnel already, tab forward, else the panel that
    holds it (`activate_retained_thread`, `focus_panel`), else `thread_tab::open_saved`
    (`thread_tab.rs:768`). Its panel branch is the one to change.
  - `thread_tab.rs`: `open_in_center` (496: `clear_base_view` and a `ThreadTab` on the same view),
    `move_to_panel` (551), `activate_for` (584), `start` (650), and the panel the tab needs
    (`open_thread` 746-749 refuses without one; `build_view` 885 takes its connection store), so
    the panel stays added and hidden.
  - `routing.rs` (`init` 31-40, `toggle` 265, `toggle_bottom_dock` 274): the house pattern,
    `register_action_renderer` + `capture_action` + `cx.stop_propagation()` in the Marley layout,
    for the Terminal Panel. `marley_workbench.rs:916` does the same for Zed's layout presets.
  - Zed's handlers it overtakes: `AgentPanel::toggle_focus` (`agent_panel.rs:1583`, registered in
    `crates/zed/src/zed.rs:906`), `toggle` (1611), `NewThread` (`agent_panel.rs:366`),
    `ToggleNewThreadMenu` (463); `OpenSettings` (403) and `NewExternalAgentThread` (409) stay
    Zed's. `ctrl-?` is bound in
    `Workspace` (`default-linux.json:713`). `has_open_project` (2955) is why the panel refuses a
    thread with no visible folder (1753, 1780, 1929, 2982, 3550, 3644).
  - The rail lists a project's threads from the store by its folders
    (`entries_for_main_worktree_path`, `entries_for_path`, `rail.rs:7688`), so a conversation
    whose tab closed keeps its row.
  - `PanelButtons` (`workspace/src/dock.rs:1391`) dispatches the panel's `toggle_action`
    (`agent_panel.rs:5132`, `ToggleFocus`) on a click: the capture sees the sparkle's click.
  - `MarleyNewItemMenu` (`workspace/src/pane.rs:4330`, asked first in the `+` builder at 4381).
  - `MarleyThreadHost::reveal` (`agent_ui.rs:520`), asked by a notification's accept
    (`conversation_view.rs:3321`) before Zed reveals the panel.
  - `harness.rs` `manager_in_front` (441) counts the Manager as read only while the panel shows
    it; with the Manager in a tab it must also read the shown thread tab.
  - `ThreadsArchiveView` (`threads_archive_view.rs:163`) and Zed's `Picker`: the picker is
    `Picker::uniform_list` with `fuzzy::match_strings`, as #735's is.
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (panes, the
  palette); `docs/orca_architecture/01-agents-and-sessions.md` §2.1 (Orca starts an agent in the
  worktree root, the root-first rule here).
- **Published material:** none beyond ACP's `session/new` and `session/load`, used as #734 and
  #736 use them.

## UI proof
`script/e2e/744-one-new-agent.sh`, under `compositor sway` (rows, `+` menus and the status bar take
clicks). #737's set-up: the scripted agent runs the Marley entry (`MARLEY_ASSISTANT_ADAPTER`) and
logs `session/new` with its `cwd` and `session/load`; a fake `claude` first on the PATH, so the
Agent CLIs list would show if it were still there; the copy's thread list emptied, as #739's run
does. The scratch project `repo` with `repo/sub`, and a terminal in `repo/sub`.

Shots:
- `744-01-row-plus`: the repo row's `+` open: New Terminal, New Browser Tab, New Agent…, and no Reopen
  Thread…, no New Agent Thread, no Agent CLIs.
- `744-02-where`: New Agent… from that `+`, Marley chosen: Where lists `repo` "this project"
  first, `sub` "this terminal's folder" second.
- `744-03-started`: Enter: a Marley thread tab in the repo; the log's `cwd` is `repo`; "first"
  answered.
- `744-04-sparkle`: Default shown, the status bar's sparkle clicked: the New Agent picker over
  Default; no right dock.
- `744-05-key`: Escape, then `ctrl-?` in the repo: the picker again; no right dock.
- `744-06-pane-plus`: the repo pane's `+` open: New Agent… above New File.

## Locked-In Decisions
- **D1 — One entry, the picker.** New Agent… replaces New Agent Thread and the Agent CLIs list in
  the row's `+`: Chad's "just 1 singular open a agent". The picker already lists both kinds
  (#735). Rejected: keeping the CLIs list (two ways to start the same CLI in one menu). Chad may
  want the CLIs list back; it is one fold to restore.
- **D2 — The project's root leads Where.** In a group with a project, the root is the first row
  wherever the picker opens, so Enter twice starts in the project. #735's D2 (the guess first) is
  kept for groups with no folder. Rejected: the root first only from a row (one picker, two
  rules).
- **D3 — The pane's `+` uses #699's hook** (D5 and D6, the opening path and the Reopen Thread
  picker, moved to #772), which keeps the hook in Zed's `pane.rs` with a user
  and its ledger row unchanged in substance. Rejected: taking the hook out of Zed (the tab bar's
  `+` is where a new tab is asked for, and Chad's "one New Agent" covers it).
- **D4 — Catch Zed's agent actions, keep Zed's button.** In the Marley layout the sparkle stays
  Zed's dock button and its action is caught, as the Terminal Panel's are (`routing.rs`). No Zed
  hunk. Rejected: `agent.button = false` in the layout's defaults with a sparkle of Marley's (that
  setting also hides the editor toolbar's agent button, `quick_action_bar.rs:713`); a hook in
  `agent_panel.rs` (a Zed touch the capture makes unneeded). Known cost: the sparkle's tooltip
  still reads "Agent Panel".
- **D7 — The panel stays,** added and hidden: a thread tab takes its connection store
  (`build_view`), and its settings page has no other home yet (#747).
- **D8 — `marley: new agent thread` leaves the palette** but keeps working from keys, so the
  palette has one New Agent. Rejected: removing the action (#734's D6 keys and #749 use it).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail row's `+` menu shall list New Agent…, and shall not list New Agent Thread or the agent CLIs. | Shot 744-01 |
| REQ-002 | WHEN the New Agent picker opens in a group with a project, Where shall list the project's root first, marked "this project", and the guess after it when it is another folder. | Shot 744-02 |
| REQ-003 | WHEN Where's first row is chosen for a thread agent, the system shall open its thread in a tab of that group, working in the project's root. | Shot 744-03 and the agent log's `cwd` |
| REQ-004 | WHILE the Marley layout is used, WHEN the status bar's sparkle is clicked, the system shall open the New Agent picker in the shown group and leave the right dock closed. | Shot 744-04 |
| REQ-005 | WHILE the Marley layout is used, WHEN `agent::ToggleFocus`, `agent::Toggle`, `agent::FocusAgent`, `agent::NewThread` or `agent::ToggleNewThreadMenu` runs, the system shall open the New Agent picker. | Shot 744-05 (`ctrl-?`); the review for the other four |
| REQ-006 | WHILE AI is on, every pane's `+` menu shall start with New Agent…. | Shot 744-06 |
| REQ-013 | WHILE Zed's layout is used, the sparkle and Zed's agent actions shall open Zed's Agent Panel as before. | The review (each capture returns without stopping propagation outside the Marley layout) |
| REQ-014 | The palette shall not list `marley: new agent thread`, and a key bound to `marley::NewAgentThread` shall still start a thread. | The review |
| REQ-015 | The change shall pass `script/gates.sh --diff`. | The gate's exit code |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, every cited line checked
  again after #743 lands.
- **P2 Code** — `agents.rs` (the root-first Where, the captures, the pane hook), `thread_tab.rs`
  (`NewAgentThread` hidden from the palette), `rail.rs` (the `+`), the in-app guide page; a review
  of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/744-one-new-agent.sh`, read every shot.
- **P4 Complete** — CHANGELOG, `docs/marley/guide.md`, the walkthrough and the architecture note
  (§21), the `pane.rs` ledger row's text (its user is now New Agent…), ledger capture (§19),
  close the ticket, archive, commit.
