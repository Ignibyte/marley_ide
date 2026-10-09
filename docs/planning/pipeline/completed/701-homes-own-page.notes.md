# Home's own page in place of Zed's Welcome page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-701-homes-own-page.md
- **Pipeline spec:** 701-homes-own-page.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09; the content the session proposed to him (Start, Recent Projects,
  Agents at Work, Configure, with New Agent Thread's entries); run by a fork, proceeding on
  defaults.
- **Classification:** feature, Marley layout, Marley crate only.
- **Recall (§18.3):** AD-679 (Chad asked to "force a home page" for Rusty: a page of cards, first
  in its group); AD-699 (an empty Rusty group shows its page, not Zed's Welcome page);
  AD-700 (Home exists from the start and takes Zed's start workspace, with the Onboarding tab Zed
  opens on a first start). The guide (#600) says a group's + has no New Agent Thread, "which need
  a folder"; whether Zed's Agent Panel runs a thread in a folderless workspace is checked in Test.
- **Discovery:** see the spec's Prior art. `Pane::add_item_inner(…, activate, Some(0), …)` adds
  without taking the active tab.

### Design
- **`home_page.rs`** (new, Marley crate): `MarleyHome` (`Item`: "Home", `IconName::ListTree`, as
  the rail's Home header), `ensure(workspace, window, cx)`, `fill(workspace, window, cx)` for the
  Home group, `init`. Cards as `RustyHome` draws them; recent projects read once at creation
  (`WorkspaceDb::recent_project_workspaces`, local ones, at most 8); the page observes the rail
  for Agents at Work.
- **`rusty/home_tab.rs`:** its `ItemRemoved`, `ActiveWorkspaceChanged` and `Groups` hooks call
  `home_page::fill` beside its own.
- **`rail.rs`:** `Rail::agents_at_work()` (from the snapshot: threads Running/Waiting/Error, agent
  terminals Working/Waiting/Failed, with project and status word) and `Rail::open_selection`.
- **`marley_workbench.rs`:** `pub mod home_page;` and its `init`.
- **File manifest:** the four above (Marley crate); `docs/marley/guide.md`; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Fresh profile; opens repo, quits; relaunches on repo2 by path; clicks Home's header | 701-01-home-page |
| 002 | Clicks repo under Recent Projects | 701-02-recent: repo open in the rail |
| 003 | Back on Home, clicks New Terminal | 701-03-terminal |
| 004 | Back on the page, clicks Marley under New Agent Thread, types a prompt the scripted agent holds | 701-04-thread; the agent log's `session/new` |
| 005 | Back on the page | 701-05-at-work: the thread listed as working |
| 006 | `pane: close all items` in Home | 701-06-emptied |

### Risks
- **A thread in a folderless workspace** may not start an external agent; then New Agent Thread
  keeps only the agents that do, and the notes say which.
- **The rail's snapshot** is read in the page's render; it is a plain read of the rail's state.

## Phase 2 — Code
- **Built:**
  - `home_page.rs` (new): `MarleyHome`, a tab titled Home (`ListTree`, as the rail's Home header),
    with five cards. `ensure` adds it first with `add_item_inner(…, activate: empty, Some(0))`;
    `fill_emptied` runs once Home's last tab closes.
    - **Start:** New Terminal goes through the rail's own path (`TerminalPanel::
      add_center_terminal`, `default_working_directory`, the launcher's factory) on the page's
      workspace. New Agent…, Open Folder…, Clone Repository… and Command Palette dispatch
      `NewAgent`, `workspace::Open::DEFAULT`, `git::Clone` and `command_palette::Toggle` from
      the page's focus handle.
    - **New Agent Thread:** `agents::thread_agents` on Home's project, each button
      `agents::start_thread` on Home's workspace.
    - **Recent Projects:** `WorkspaceDb::recent_project_workspaces`, local ones, at most 8, read
      once at creation; a click calls `open_workspace_for_paths(OpenMode::Activate, …)`.
    - **Agents at Work:** the window's rail, found in render from the `MultiWorkspace`'s sidebar
      and observed from then on; `Rail::agents_at_work` lists the rows, and a click calls
      `Rail::open_selection`.
    - **Configure:** `OpenSettings`, `OpenSettingsAt { path: "marley.layout" }`, `OpenKeymap`,
      `Extensions`, `OpenGuide`.
  - `rail.rs`: `AtWork`, `agents_at_work` (threads not Done, agent terminals not Idle) and
    `open_selection`; `AgentStatus` imported.
  - `groups.rs`: `is_home_workspace`.
  - `rusty/home_tab.rs`: `card`, `row` and `muted` are `pub(crate)` for the page. #699's
    `ItemRemoved` hook also calls `home_page::fill_emptied`, and `fill_shown` ensures Home's page
    when the shown workspace is Home.
  - `marley_workbench.rs`: `pub mod home_page;`.
  - `docs/marley/guide.md`: Home's page.
- **Deviation:** no `home_page::init`. The page needs no action and no hook of its own, since
  #699's hooks carry it.
- **Hook note:** as in #700, the fork's phase hooks read the parent's transcript; edits went
  through Bash.
- **Review of the diff:**
  - `ensure` runs inside the workspace's update, and from `fill_shown` inside the
    `MultiWorkspace`'s subscription callback. So `MarleyHome::new` reads neither: it takes the
    `Fs` from `ensure`, and the rail is looked up in `render`, where neither is leased.
  - Clicks run outside updates. The workspace and the rail are updated through weak handles,
    and an action dispatch starts from the page's own focus handle, so it reaches Home's
    workspace even when nothing else holds focus (L-700).
  - A Home page closed beside other tabs stays closed until Home is shown again or empties, so
    its × works.
- **Gate:** `701-gate-1.log` RED on clippy: `clone_on_ref_ptr`, two `unused_self`, and one
  `needless_pass_by_ref_mut`. All fixed; `701-gate-2.log` is GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/701-homes-own-page.sh`, under `compositor sway`. On a fresh profile
  it opens repo and quits (clicking Home's header first, L-700), then relaunches by path on repo2,
  so repo is a recent project that is not open. A stand-in `claude`, a Python script first on the
  terminals' PATH through the scenario's `.bashrc`, prints a line every half second for a minute.
  No real Claude Code runs (PR-687).
- **Runs and what they found:**
  - `shots-701a` (**red**): after the path launch the rail listed "**Home 2**". The last
    session's Home record stays pending, since a path launch restores nothing (L-601). After
    `SETTLE`, #700's rule made a Home again, and `groups::make` numbered every name but Rusty's
    against the taken ones. **Fix:** Home keeps its name, as Rusty does; only named groups are
    numbered. Also, the relaunch's trust prompt took no key (L-700), so the scenario clicks
    Trust and Continue.
  - `shots-701c`: the page showed its five cards, but a recent project's name was squeezed to
    "rep…" by its long folder. **Fix:** the name keeps its width and the folder is what gets cut.
  - `shots-701d` (**red, a crash**): clicking a recent project panicked with "cannot read
    MarleyHome while it is already being updated". The click was a `cx.listener`, which leases
    the page; inside it the workspace's `open_workspace_for_paths` read every item's
    `is_dirty`, the page among them. **Fix:** every handler on the page that updates the
    workspace is a plain closure over a weak handle, as the rail's are.
  - `shots-701f` (**red, a design change**): the Marley button under New Agent Thread opened
    Home's Agent Panel on "Choose one of the options below to use the Agent Panel: Open Project /
    Clone Repository". Zed's Agent Panel runs no thread in a folderless workspace, which is why
    the rail's + leaves New Agent Thread out of groups (#600). **Change:** the card became **New
    Agent**, a button per agent CLI on the PATH, each started in a terminal in Home's home folder
    (`agents::start_cli`). The Start card's "New Agent…" picker went with it.
  - `shots-701h`: a bash-script stand-in was not recognized as Claude Code. #519's stand-in is
    Python, which the rail names by its file, so this one is too. In `shots-701i` the close-all
    met Marley's guard for a working agent (#550), and the scenario clicks Close.
  - In `shots-701i` the row's title was the shell's title (user@host:path). **Fix:** the row's
    status names the agent, "Claude Code · working", as the rail's row does.
- **Gates after the fixes:** `701-gate-3.log` (the name), `-4` (the recent row), `-5` RED on
  clippy (a missing `;`), then `-6` and `-7`: GATE GREEN [diff].
- **Final run (`shots-701j`): every shot shows its criterion; no panic in the logs.**
  - **701-01-home-page (REQ-001):** Home's header clicked: the Home tab first and only. START
    lists New Terminal, Open Folder…, Clone Repository… and Command Palette. NEW AGENT lists
    Claude Code, Codex, Gemini CLI and OpenCode. RECENT PROJECTS lists repo2 and repo with their
    folders, AGENTS AT WORK says "No agent at work.", and CONFIGURE lists Settings, Marley
    Settings, Keymap, Extensions and Marley Guide.
  - **701-02-recent (REQ-002):** repo clicked: repo is open and shown, its terminal under it in
    the rail, between Home and repo2.
  - **701-03-terminal (REQ-003):** New Terminal: a "<user> — bash" tab in Home, at `~`, beside
    the Home tab.
  - **701-04-agent (REQ-004):** Claude Code: a terminal in Home running `claude`, the stand-in
    printing its lines. The rail reads it as "Claude Code · working", with the agent bar under
    it.
  - **701-05-at-work (REQ-005):** back on the page, AGENTS AT WORK lists that terminal: its title,
    "Home" and "Claude Code · working".
  - **701-06-emptied (REQ-006):** `pane: close all items`, Close at the guard: Home shows its page
    as its only tab, and the toast offers Undo for the closed agent. No Welcome page.
  - **REQ-007:** the gate.
  - The run reports Chad's Hyprland untouched.
- **Not reached by the scenario:** the Configure buttons and Open Folder…, Clone Repository… and
  Command Palette. Each dispatches Zed's or Marley's own action from the page's focus handle, as
  the welcome page's buttons do; a run would show Zed's own dialogs.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: Home's own page); `docs/marley_architecture/
  marley_workbench.md` ("Home's own page"); the slice line in `docs/marley/workbench-shell.md`; the
  guide's Home paragraph (Phase 2, its New Agent line corrected in Test). No Zed path touched.
- **Knowledge appended:** F-claude-701-a-listener-on-a-tab-updated-its-workspace-and-panicked-001,
  F-claude-701-a-path-launch-named-the-new-home-group-home-2-001,
  PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001,
  L-claude-701-zeds-agent-panel-needs-a-project-folder-001,
  L-claude-701-a-fake-agent-cli-is-a-python-script-001.
- **Cut, for the main session:** New Agent Thread on Home, which Zed's Agent Panel can't run
  without a project folder, became New Agent (the agent CLIs). Agents at Work shipped.
- **Brain:** no `rusty` MCP server in this repository's sessions; the decisions are in the
  ledger.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate at commit:** `701-gate-8.log`, GATE GREEN [diff].
