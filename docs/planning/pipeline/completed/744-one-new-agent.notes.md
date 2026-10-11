# One New Agent, and Reopen Thread… — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-744-one-new-agent.md
- **Pipeline spec:** 744-one-new-agent.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, the tabs-everywhere shelf
  (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`): "i wonder if the experience
  becomes just 1 singular open a agent for the left pane and the sparkle. Meaning if its a project
  then default to the project folder chosen but allow to switch. Remove New Agent THread with just
  New Agent. add a New Agent from Thread or something that restarts", then "my thought is that you
  would re open it". Of Zed's Agent Panel: "The agent pane inside zed still has the issue of
  needing to be in a project."
- **Classification:** feature, Marley crate only (`marley_workbench`). No Zed hunk: the pane `+`
  uses #699's hook, and Zed's agent actions are caught in the capture phase.
- **Order:** after #743, which removes the Rusty group, renames Home to Default and drops the only
  user of `MarleyNewItemMenu`. #743 is in its Code phase and edits `rail.rs`; every `rail.rs` line
  below is at 19dda499a2 and moves.
- **Recall (§18.3):**
  - AD-claude-450: New Agent is a picker behind the Marley keymap, in both layouts. #735 gave it
    Where; its D2 put the guess first ("Enter twice starts the agent where you are"). This
    ticket keeps that for groups with no folder and puts the project's root first in a project.
  - AD-claude-734 and #734's D1: Marley's entry points start threads in tabs; "the panel's own
    `+` keeps opening threads in the panel". With the panel no longer shown in the Marley layout,
    that clause has nothing left to apply to there.
  - AD-claude-737: the Threads page over Zed's store, a click that brings an open conversation
    forward; its D2, one place per conversation (#697), is the rule the funnel keeps.
  - AD-claude-685: the manager talks in Zed's Agent Panel over ACP. Since #734 a tab hosts the
    same `ConversationView`, so the Manager in a tab is still Zed's thread view; only its host
    moves.
  - AD-claude-699: `MarleyNewItemMenu`, a Marley hook in Zed's `pane.rs`, asked with the pane's
    workspace.
  - PR-claude-735 / F-claude-735 (both): code inside a workspace's update reads that workspace by
    reference, and a picker's confirm defers any modal swap. Reopen Thread…'s confirm and the
    rail's row click both reach `open_thread`, which reads every workspace: they defer.
  - PR-claude-701: handlers on a tab that update its workspace are plain closures over a weak
    handle.
  - PR-claude-697: a hosted thread view focuses its message editor; the tab already does.
  - Brain decision "Marley-layout terminals route to the center through a task provider and
    capture listeners, with no Zed hunk" (2026-10-06): the capture pattern this ticket reuses.
- **Discovery** (lines at 19dda499a2):
  - `crates/marley_workbench/src/agents.rs`: `NewAgent` (677), `new_agent` (700), `show_picker`
    (726), `places_for` (751), `guess` (788), `Place` (829), `NewAgentPicker::new` (888),
    `Start::run_in` (1041), `confirm` (1252), `start_thread` (192, only `open_manager_thread`
    calls it), `init` (649).
  - `crates/marley_workbench/src/rail.rs`: `load_thread` (371), `show_thread` (395),
    `open_manager_thread` (1971, `agents::start_thread` at 1995), `activate_workspace` (2553),
    `open_thread` (2823), `open_listed_thread` (2834), `open_thread_with` (2866),
    `new_agent_picker` (3009), `new_agent_thread` (3025), the Archived Threads submenu (5066-5108),
    `render_project_menu` (5110; New Agent… 5158, New Agent Thread 5167), `agent_menu` (5191),
    `agent_cli_entries` (5221), `worktree_agent_entries` (5253), `render_thread_row` (5373, Open
    in Center 5457), the agent-server subscriptions that named the New Agent Thread entries (216).
  - `crates/marley_workbench/src/thread_tab.rs`: `NewAgentThread` (78), `init` (126),
    `tab_extra_context_menu_actions` (230, Move Thread to Panel), `open_in_center` (496),
    `open_thread_in_center` (529), `move_to_panel` (551), `activate_for` (584, the `reveal` Marley
    sets), `start` (650), `open_thread` (738, panel needed at 746), `open_saved` (768),
    `build_view` (876, connection store at 885), `default_folder` (936).
  - `crates/marley_workbench/src/threads_page.rs`: `open_thread` (310), its panel branch
    (334-351), `Row` (83) and `sections` (126), `matches` (283).
  - `crates/marley_workbench/src/routing.rs` (31-40, 265-284): the capture pattern.
  - `crates/marley_workbench/src/harness.rs` `manager_in_front` (441).
  - Zed: `agent_panel.rs` `init` (363: `NewThread` 366, `OpenSettings` 403,
    `NewExternalAgentThread` 409, `ToggleNewThreadMenu` 463), `toggle_focus` (1583), `toggle`
    (1611), `has_open_project` (2955), `icon`/`toggle_action` (5124-5134); `zed.rs:906`;
    `zed_actions::assistant::{Toggle, ToggleFocus, FocusAgent}` (`zed_actions/src/lib.rs:658`);
    `workspace/src/dock.rs` `PanelButtons::render` (1391, the click at 1539-1546);
    `workspace/src/pane.rs` `MarleyNewItemMenu` (4327-4347, asked at 4381);
    `agent_ui.rs` `MarleyThreadHost` (520); `conversation_view.rs` reveal (3316-3326).
  - Guide (#743 is editing it, so find these by text): `docs/marley/guide.md` (the `+` table's
    New Agent Thread row, the group `+` paragraph, "The Marley agent", "Start one in a tab", Open
    in Center and Move Thread to Panel); the in-app page (the `+` table, the Agent Panel steps,
    Marley's, Rusty's and the Manager's New Agent Thread lines); `docs/marley/walkthrough.md`
    (two New Agent Thread steps). `grep -n 'New Agent Thread'` finds them all.

### Design
- **`agents.rs`.**
  - `places_for`: when `workspace.root_paths(cx)` has a first root, it leads as
    `Place::folder(root, "this project")`; the guess follows unless it is the same folder; the
    rest is unchanged. `guess` keeps its order for the second row and for groups with no folder.
  - The captures, registered from `init` with `register_action_renderer` +
    `capture_action`, each `if marley_layout(cx) { cx.stop_propagation(); … }`:
    `zed_actions::assistant::{ToggleFocus, Toggle, FocusAgent}`, `agent_ui::NewThread` and
    `agent_ui::ToggleNewThreadMenu` → `show_picker(workspace, None, …)`. `crate::marley_layout`
    is the switch. `agent_ui::NewExternalAgentThread` is not caught: only the panel dispatches it
    (`agent_panel.rs` 6059, 9368), and its `agent` field is private with no getter, so reading it
    would take a Zed hunk.
  - The pane hook: `cx.set_global(MarleyNewItemMenu(Arc::new(new_item_entries)))` from `init`;
    `new_item_entries` adds New Agent… and Reopen Thread… and a separator while AI is on, each a
    plain closure over the weak workspace that updates it (`show_picker`, `reopen_thread`).
  - `start_thread` goes with its last caller.
- **`threads_page.rs`.**
  - `ReopenThread` action (`marley` namespace) registered on every workspace; `reopen(workspace,
    window, cx)` opens `ReopenThreadPicker` in the workspace's modal layer.
  - `ReopenThreadPicker` / its delegate: the rows `sections` gives, archived left out (a shared
    `conversations(cx) -> Vec<Row>` the page and the picker both read), newest first;
    `fuzzy::match_strings` over "title folder"; each entry draws the agent's icon, the title, "agent
    · folder" muted and the time at the end, and "open" when a tab shows it. Confirm dismisses,
    then `window.defer` → `open_thread(&workspace, thread_id, …)`.
  - `open_thread`: its panel branch, in the Marley layout, calls the workspace-local half for that
    member instead of `activate_retained_thread` + `focus_panel`; Zed's layout keeps today's
    branch.
- **`thread_tab.rs`.**
  - `show_here(workspace: &mut Workspace, thread_id, window, cx) -> bool`: its tab forward
    (`activate_for`); else, when its panel holds the view (`active_conversation_view` or
    `is_retained_thread`), the view into a new `ThreadTab` of this workspace: the active one through
    `open_in_center`'s steps (`clear_base_view` first), a retained one hosted as it is (the panel
    does not draw it, and `ThreadTab::new`'s subscription closes the tab if the panel shows it
    again); else `false`.
  - `init`: `MarleyThreadHost.reveal` becomes `reveal_for`: `activate_for` in Zed's layout,
    `show_here` in the Marley layout.
  - Move Thread to Panel: `tab_extra_context_menu_actions` returns nothing in the Marley layout;
    `MoveThreadToPanel` and `OpenThreadInCenter` are hidden from the palette there
    (`CommandPaletteFilter`, refreshed when the layout setting changes), and `NewAgentThread` is
    hidden in both.
- **`rail.rs`.**
  - `render_project_menu`: New Agent… (as today), Reopen Thread… (`reopen` in the row's workspace
    after `activate_workspace`), New Agent in Worktree, launch entries; `agent_menu`,
    `agent_cli_entries`, `new_agent_thread` and the agent-server subscriptions that only fed the
    submenu go (check `snapshot` for other readers first).
  - `open_thread_with`: inside `in_workspace`, `window.defer` → `threads_page::open_thread` for
    the row's workspace; an archived thread is unarchived in the store first (#616). Zed's layout
    has no rail, so no layout check is needed here.
  - `render_thread_row`: Open in Center leaves the menu; `open_listed_thread`'s `center` flag goes
    with it.
  - `open_manager_thread`: a listed Manager thread opens through the funnel; none →
    `thread_tab::start(workspace, Some(MANAGER_ENTRY), None, …)` in the shown workspace.
- **`harness.rs`.** `manager_in_front` is also true when the shown workspace's active item is a
  thread tab on the Manager entry (a `thread_tab` helper beside `activate_agent`).
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): the `+` table, the
  sparkle, Reopen Thread…, the Marley and Rusty and Manager lines that name New Agent Thread.
- **File manifest:** `agents.rs`, `threads_page.rs`, `thread_tab.rs`, `rail.rs`, `harness.rs`,
  `guide/index.html` (all Marley crate); `script/e2e/744-one-new-agent.sh`; at Complete,
  `docs/marley/guide.md`, `docs/marley/walkthrough.md`, `CHANGELOG.md`, the architecture note,
  and the `pane.rs` row of `docs/marley/zed-touchpoints.md` (its "why" column names the Rusty
  group's screens; it becomes New Agent… and Reopen Thread…). No Zed crate is edited, so no new
  ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Clicks the repo row's `+` | 744-01-row-plus: the menu without New Agent Thread or Agent CLIs (the fake `claude` is on the PATH) |
| 002 | A terminal in `repo/sub` active; `+` → New Agent… → types Marley, Enter | 744-02-where: `repo` "this project" first, `sub` second |
| 003 | Enter; asks "first" | 744-03-started: a Marley tab in the repo; the log's `cwd` is `repo` |
| 004 | Clicks Default's row, then the status bar's sparkle | 744-04-sparkle: the picker over Default, no right dock |
| 005 | Escape; clicks the repo row; `ctrl-?` | 744-05-key: the picker, no right dock |
| 006 | Escape; clicks the pane's `+` | 744-06-pane-plus: New Agent…, Reopen Thread… above New File |
| 007 | Closes the Marley tab; row `+` → Reopen Thread… | 744-07-reopen: the conversation's entry |
| 008 | Enter | 744-08-reopened: the tab with "first" and its answer; `session/load` logged |
| 009 | Reopen Thread… → the same entry | 744-09-again: the same tab in front, one Marley tab |
| 010 | Closes the tab; clicks the thread's rail row | 744-10-row: a tab in the repo, no right dock |
| 011 | Right-clicks the thread tab | 744-11-tab-menu: no Move Thread to Panel |
| 005 (rest), 011 (panel), 012 (palette), 013, 014 | none | The review: the other captured actions, a panel-held thread (only Zed's own panel paths put one there: a restored panel, which #747 handles), the palette filters, Zed's layout passing through, the key for `NewAgentThread` |

Coordinates (the sparkle, the `+` buttons, the rows) are found by a first run that stops at the
first shot, as #739's `STOP_AT_BAR`.

### Risks
- **Capture order.** The workspace root's capture listeners run before Zed's bubble handlers on
  the same root (`gpui` `window.rs` 6502-6546), and the dock button's click dispatches from the
  dock's focus handle inside the workspace, so the capture sees it. `routing.rs` relies on the
  same for the Terminal Panel; check the sparkle in the first run.
- **The sparkle's tooltip** still reads "Agent Panel" (D4). Chad will redraw the icons; a name
  change would be a Zed touch.
- **Size.** Large: the menus, the captures and the pane hook are small; the picker and the funnel
  are the bulk. If it will not fit, it splits cleanly: (a) one New Agent (the `+` menus, Where's
  first row, the captures, the pane hook) and (b) Reopen Thread… with the funnel (rail rows,
  Archived Threads, the Threads page, the Manager, `reveal`, Move Thread to Panel).
- **A retained view hosted in a tab while the panel keeps it** relies on `ThreadTab::new`'s
  `ActiveViewChanged` subscription to close the tab if the panel ever shows it; Zed's own panel
  paths (Out, #747) can still do that until #747.
- **The scripted agent's `session/load`** must replay "first" for 744-08, as #736's does.

### Promotion (2026-10-11, after #743 landed as 309370d75e)
- **Split.** The ticket was L; its notes' proposed split is taken. #744 keeps one New Agent…
  (Where's first row, the row's `+`, the pane's `+`, the sparkle and Zed's agent actions caught,
  `marley: new agent thread` off the palette). Reopen Thread…, the one opening path, the rail's
  thread rows, Archived Threads, the Manager, `reveal`, Move Thread to Panel and Open in Center go
  to TICKET-772, which keeps this file's design for them.
- **Seams re-checked at 309370d75e:** `agents.rs` `init` 649, `new_agent` 700, `show_picker` 726,
  `places_for` 751, `guess` 788 (unchanged); `rail.rs` `new_agent_picker` 3052, `new_agent_thread`
  3068, `render_project_menu` 5144 (New Agent… 5192, New Agent Thread 5201), `agent_menu` 5225,
  `agent_cli_entries` 5255; the empty-space menu's CLI entries (4273) call `Rail::new_agent`, which
  stays. `agent_server_subscriptions` (217, resubscribed at 2526 from `Watched::agent_servers`)
  only refreshed the rail for the submenu's names; the submenu read `thread_agents` when it
  opened, and no row reads the stores, so the subscriptions go with it.
- **Design for this half:** `places_for` puts `workspace.root_paths(cx).first()` first as "this
  project", then the guess when it differs; `agents::init` registers the captures
  (`zed_actions::assistant::{ToggleFocus, Toggle, FocusAgent}`, `agent_ui::{NewThread,
  ToggleNewThreadMenu}` → `show_picker`, Marley layout only, `cx.stop_propagation()`) and sets
  `MarleyNewItemMenu` to an entries function adding New Agent… and a separator while AI is on;
  `rail.rs` drops New Agent Thread, the Agent CLIs list and the subscriptions; `thread_tab.rs`
  hides `NewAgentThread` from the palette (`CommandPaletteFilter`); the in-app guide page and the
  docs follow.
- **Visual check:** shots 744-01 to 744-06 of the spec's UI proof.

## Phase 2 — Code
- **Built:** `agents.rs`: `places_for` puts the workspace's first root first as "this project" and
  the guess after it unless it is the same folder; `init` (now `&mut App`) registers capture
  listeners for `agent::{ToggleFocus, Toggle, FocusAgent, NewThread, ToggleNewThreadMenu}` through
  one generic `picker_in_place_of`, which in the Marley layout stops propagation and opens
  `show_picker`, and sets `MarleyNewItemMenu` to `new_item_entries` (New Agent… with
  `marley::NewAgent`'s key shown, then a separator, while AI is on). `rail.rs`: the row's `+`
  loses New Agent Thread and the Agent CLIs list; `agent_menu`, `agent_cli_entries` and
  `new_agent_thread` go; the empty-space menu keeps its CLI entries (`Rail::new_agent`).
  `thread_tab.rs`: `NewAgentThread` hidden from the palette in each new workspace's setup. The
  in-app guide page: the `+` table gains New Agent… and loses New Agent Thread and Agent CLIs; the
  Agent Panel, Marley, Rusty and Manager steps name New Agent…. `docs/marley/guide.md` and the
  walkthrough follow.
- **Deviation:** the agent-server subscriptions stay. The plan read them as feeding only the
  submenu; `agents::thread_icon` reads each project's agent-server store for the thread rows'
  icons, so the subscription's refresh keeps a row's icon right when a server's icon arrives. Its
  doc comment now says so.
- **Review:** the `+` entry's action argument only shows the key (`ContextMenu::entry` keeps the
  handler as the one thing a click runs); the capture listeners return without stopping
  propagation outside the Marley layout, so Zed's layout reaches the panel (REQ-013); `show_picker`
  runs inside the workspace's update as `new_agent` already does.
- **Gate:** first run red on one clippy finding (`too_long_first_doc_paragraph` on `agents::init`'s
  doc) and the receipt (the scenario was written during the run); the doc split into a line and
  a paragraph. The final run is in Phase 4.

## Phase 3 — Test
- **Scenario:** `script/e2e/744-one-new-agent.sh`, `compositor sway`, #735's fixtures (a fake
  `claude` first on the PATH, the Marley entry on a scripted ACP agent logging `session/new`'s
  `cwd`). Four runs: the first clicked the repo's `+` at #743's y (182), but with no Home row yet
  the repo sat at 136, and its terminal had not appeared before `cd sub` (a start-up timing flake,
  gone on later runs); the second and third showed that once Default holds its Home row the repo
  moves to 182, and that a key sent from the rail never reaches the workspace's capture listeners
  (the rail sits outside the workspace's element), so the run clicks into the center first; wtype
  sends Ctrl+? as Ctrl+Shift+/, which the thread's editor took for Ctrl+/ and typed a comment, so
  the fourth runs the same actions from the palette. The fourth exits 0 with both checks passing
  (`session/new` logged; its `cwd` is `repo`, not `repo/sub`).
- **Shots (fourth run):**
  - `744-01-row-plus`: the repo row's `+` lists New Terminal, New Browser Tab, New Agent… and
    nothing else: no New Agent Thread, no Agent CLIs although the fake `claude` is on the PATH.
    REQ-001.
  - `744-02-where`: New Agent… → Marley: "Where should Marley start?" lists `repo` "this project"
    first and `sub` "this terminal's folder" second, then recent projects and Browse…. REQ-002.
  - `744-03-started`: Enter on the first row: a Marley thread tab in the repo beside `sub — bash`,
    its row under repo, `Marley · idle`; the log's `cwd` is the repo. REQ-003.
  - `744-04-sparkle`: Default shown, the status bar's sparkle clicked: "Start an agent…" over
    Default's Home page; no Agent Panel; the sparkle's tooltip still reads "Agent Panel Ctrl-?"
    (D4's known cost). REQ-004.
  - `744-05-key` and `744-05b-new-thread`: in the repo with the thread focused, `agent: toggle
    focus` (the sparkle's and Ctrl+?'s action) and then `agent: new thread` from the palette each
    open the New Agent picker. REQ-005; `Toggle`, `FocusAgent` and `ToggleNewThreadMenu` go
    through the same listener (the review).
  - `744-06-pane-plus`: the repo pane's `+`: New Agent… with Ctrl-Alt-N at its head, a separator,
    then Zed's New File, Open File, Search Project, Search Symbols, New Terminal, New Center
    Terminal. REQ-006.
- **Reviewed, not shot:** REQ-013 (Zed's layout: each listener returns without stopping
  propagation when `marley_layout` is false); REQ-014 (`NewAgentThread` hidden through
  `CommandPaletteFilter` at each workspace's setup; its key binding still dispatches).
- **Pre-existing, not in scope:** the run log's headless keymap, Vulkan loader and llama.cpp lines;
  the right dock opening on a fresh project with the project panel (#747).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed: one New Agent…); the architecture note
  (`marley_workbench.md`, "One New Agent…"); the slice line in `workbench-shell.md`; the `pane.rs`
  row of `zed-touchpoints.md` names New Agent… as its user (the hook itself is unchanged); the
  guide, the walkthrough and the in-app page came with Phase 2.
- **Knowledge appended:** L-claude-744-a-key-sent-from-the-rail-misses-the-workspaces-listeners-001.
  No bug reached a run.
- **Brain:** no consultation opened for this ticket; the decision is #735's and Chad's
  2026-10-11 words, recorded in the shelf note.
- **Ticket:** closed; its BACKLOG row left at promotion; TICKET-772 carries the other half.
- **Gate:** GATE GREEN [diff], 17 passed, on the tree committed.
