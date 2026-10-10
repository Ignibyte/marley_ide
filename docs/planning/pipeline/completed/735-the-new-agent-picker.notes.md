# The New Agent picker — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-735-the-new-agent-picker.md
- **Pipeline spec:** 735-the-new-agent-picker.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #450 made the picker (`agents.rs` `NewAgentPicker`), #532 the permission modes it starts CLIs with, #701 Home's New Agent card.
  - PR-claude-695-a-scenario-chooses-a-menu-entry-from-home-001: menus open with nothing chosen.
  - `Workspace::prompt_for_open_path` is already used by `agent_bar.rs` and `rusty/import.rs`; Zed's picker returns one path, file or folder.
  - The project panel has no hook for another crate's menu entry: the touch is three hunks beside Open in Terminal.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** no active pipeline; README marker present; cargo idle.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Seams re-verified:**
  - `agents.rs`: `NewAgentPicker` / `NewAgentDelegate` (695/792), `Start::{Thread, Cli}` and
    `Start::run` (`start_thread` into the panel, `start_cli`); `launch_input` (376) and
    `start_in_terminal(workspace, Some(dir), Some(kind), Some(line), joining, …)` (410), used so by
    `launch.rs:433-452`.
  - `NewAgent` is a unit action in `marley_workbench.rs`'s `actions!` (151), bound
    `secondary-alt-n` in `keymap.json`. gpui builds a binding with no arguments from `{}`
    (`action.rs:363`), so a data action with only optional fields keeps that binding.
  - `Workspace::prompt_for_open_path` (`agent_bar.rs:58` uses it with `DirectoryLister`).
  - `WorkspaceDb::recent_project_workspaces` (`home_page.rs:106`, local ones only).
  - `Terminal::working_directory()` (`terminal.rs:3531`): `None` for a remote terminal.
  - `rail.rs` `terminal_context_menu` (5544): Rename, Move to Project, Close.
  - `project_panel.rs` `deploy_context_menu` (1180-1215, Open in Terminal at 1202) and
    `open_in_terminal` (4143: a file's folder for a file); no hook for another crate's entry.
  - `home_page.rs` `render_agents` (238): the CLI buttons; its doc says Zed's panel runs no thread
    in a folderless workspace, which #734 made false.

### Design
- **`marley_workbench.rs`** (Marley): `NewAgent` leaves `actions!` for `agents.rs` as
  `NewAgent { folder: Option<PathBuf> }` (`Action`, read through a fields struct as
  `rusty::OpenPage`), re-exported as `crate::NewAgent`.
- **`agents.rs`** (Marley):
  - `new_agent` passes `action.folder` to `NewAgentPicker::new`.
  - The delegate gains `stage: Stage { Agent, Where(Start) }`, `fixed: Option<PathBuf>`, and
    `places: Vec<Place>`; `Place { Folder { name, path, note }, Browse }`.
  - Confirming an agent: with `fixed`, run there; else build the places (`guess`, the window's
    open projects, recent local projects, Browse…), switch to `Where`, clear the query and
    refresh. Recent projects load in the background when the picker opens.
  - Confirming a place: a folder runs there; Browse… dismisses, asks Zed's path prompt for a
    folder (a file means its folder), then runs.
  - `run_in(start, folder)`: a thread → `thread_tab::start(Some(agent), Some(folder))`; a CLI →
    `launch_input` + `start_in_terminal` with the folder.
  - `guess(workspace, cx)`: the active item's project folder (its worktree root), else the active
    terminal's `working_directory()`, else `thread_tab::default_folder`. A terminal comes first
    when it is the active item.
  - `pub(crate) fn open_picker(workspace, folder, window, cx)` for the rail and Home.
- **`rail.rs`** (Marley): New Agent… at the head of the `+` menu's agent part; Open Agent Here… on a
  terminal row, with the terminal's folder.
- **`home_page.rs`** (Marley): a New Agent… button first in the NEW AGENT card; its doc comment
  corrected.
- **`crates/project_panel/src/project_panel.rs`** (Zed): `OpenAgentHere` in its `actions!`; the
  entry after Open in Terminal; `open_agent_here`, which takes the folder as `open_in_terminal`
  does and dispatches `cx.build_action("marley::NewAgent", Some(json!({"folder": …})))`; its
  registration beside `open_in_terminal`'s. Row in `zed-touchpoints.md` first.
- **The guide:** the picker's Where step, Open Agent Here, the line about threads needing a folder.
- **The scenario** is written before the gate, so the gate's fingerprint covers it once.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002, 007 | `cd sub` in the repo terminal; Ctrl+Alt+N; "Claude Code" (CLI) | 735-01-where: Where with the guess `repo/sub` first, repo, recent projects, Browse… |
| 004 | Ctrl+Alt+N; Marley (thread); Enter on the guess | 735-03-thread: a thread tab, the log's `cwd` `repo/sub` |
| 003 | Ctrl+Alt+N; Claude Code (CLI); Browse…; `other/` confirmed | 735-02-browse: a terminal with the fake `claude` in `other/` |
| 005 | The repo terminal's row → Open Agent Here… → Claude Code | 735-04-here-terminal: a new `claude` terminal in `repo/sub` |
| 006 | The project panel's `docs` → Open Agent Here → Claude Code | 735-05-here-panel: a `claude` terminal in `repo/docs` |
| 007 | The project's `+` menu open | 735-06-plus: New Agent… heading the agent part |

### Risks
- **The path prompt's confirm** may take the highlighted child rather than the typed folder; the
  first run's shots settle the keys.
- **A remote terminal** gives no working directory; its row's Open Agent Here… falls back to the
  guess.

## Phase 2 — Code
- **Checklist** (no TaskCreate here): ledger row ✓, `project_panel.rs` ✓, `agents.rs` ✓,
  `marley_workbench.rs` ✓, `rail.rs` ✓, `home_page.rs` ✓, `agents_tests.rs` (compiles) ✓, guide ✓,
  scenario (before the gate) ✓, gate ✓.
- **Built:**
  - `project_panel.rs` (Zed): `OpenAgentHere`, its entry after Open in Terminal, `open_agent_here`
    (the entry's folder, a file's parent) dispatching `marley::NewAgent { folder }` built by name,
    and its registration beside `open_in_terminal`'s; the ledger row first.
  - `agents.rs`: `NewAgent { folder }` (moved out of `marley_workbench.rs`'s `actions!`, re-exported
    as `crate::NewAgent`, read through `NewAgentFields`); `show_picker`; `places_for` (the guess,
    the window's open projects, Browse…); `guess`; `shown_path`; `Place`; recent projects loaded in
    the background (`load_recent`, eight at most, local only); the delegate's `fixed`, `places`,
    `chosen`; `Start::run_in` (a thread through `thread_tab::start`, a CLI through `launch_input` and
    `start_in_terminal`) in place of `run`; `browse` / `prompt_and_run`; Where's rows (folder icon,
    name, path, note) and its placeholder.
  - `rail.rs`: `new_agent_picker`; New Agent… in every `+` menu; Open Agent Here… on a terminal row.
  - `home_page.rs`: New Agent… first in the NEW AGENT card.
  - `agents_tests.rs`: `NewAgent::default()` where the unit action was dispatched, and `Action`
    unqualified now that the module imports it. Tests that confirm a Thread choice and expect a
    panel thread no longer describe the picker (it opens a tab now); no gate runs them (§7), and
    they keep building.
  - The guide: the `+` table, the picker's Where, Open Agent Here, Home's card.
- **Found and fixed before the gate** (runs `shots-735a`, `shots-735b`, both crashes):
  - `places_for` ran inside the workspace's own update and read every workspace of the window,
    itself included: gpui panicked, "cannot read Workspace while it is already being updated".
    The current workspace is now read through its reference (`current` is its id).
  - Browse… opened Zed's path prompt from inside the picker's confirm; the modal layer, hiding the
    picker for the prompt, read the picker being updated, and gpui panicked. Browse now runs from
    `window.defer`, outside any update.
  - Where's placeholder stayed "Start an agent…": the picker reads it once; `refresh_placeholder`
    now runs with the switch.
- **Review of the diff:** REQ-001/002 `places_for` + `guess`; REQ-003 `prompt_and_run`; REQ-004
  `run_in` → `thread_tab::start`; REQ-005 the row's `working_directory()`; REQ-006 the panel's
  handler; REQ-007 the menus. Re-entrancy: the two panics above; Home's button is a plain closure
  over a weak handle (PR-claude-701). Upstream: four additive hunks in one Zed file, each with a
  `// Marley:` comment, and its row. Provenance: `open_agent_here` takes the folder as Zed's
  `open_in_terminal` does, in Zed's own crate.
- **Gate:** `735-gate-1.log` RED, gate:2 only: three `needless_pass_by_ref_mut` (`load_recent`'s
  `cx`, `run_in`'s and `browse`'s `window`), made shared references. `735-gate-2.log`: GATE GREEN
  [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/735-the-new-agent-picker.sh`, under `compositor sway`, Zed's system path
  prompts off (D5). A fake `claude` first on the PATH prints and logs its folder; the Marley entry
  runs #734's scripted agent. Written before the gate; the runs before it (`shots-735a` to
  `shots-735c`) found the two panics and the placeholder (Phase 2).
- **The Test phase's run (`shots-735-test`), after `just build` and `735-gate-2.log` green: every
  check passes.**
  - **735-00-layout:** the repo's terminal after `cd sub`, its row "sub — bash"; the project panel
    lists docs, sub, README.md.
  - **735-01-where (REQ-001, REQ-002):** after Ctrl+Alt+N → Marley, the picker asks "Where should
    Marley start?" and lists sub (this terminal's folder) first, then repo (open project), three
    recent projects from the profile's history, and Browse….
  - **735-03-thread (REQ-004):** Enter on the guess: a "New Agent Thread" tab in repo, its message
    box "Message Marley", its row under repo; the check: the session's `cwd` is `repo/sub`.
  - **735-02a-prompt:** Browse… opened Zed's own folder prompt in the run's window, the typed
    `…/other/` with "open this directory" first and `notes.md` under it.
  - **735-02-browse (REQ-003):** a terminal "other — sleep 600" in repo, the fake printing "fake
    claude in …/other"; the check: its log holds `other/`.
  - **735-04a-row-menu:** the repo terminal row's menu: Rename, Open Agent Here…, Move to Project,
    Close.
  - **735-04-here-terminal (REQ-005):** with no Where step, a terminal "sub — sleep 600", "fake
    claude in …/repo/sub"; the checks: a second start, in `repo/sub`.
  - **735-05a-panel-menu:** `docs`'s menu in the project panel: … Open in Terminal, Open Agent Here
    (chosen), Find in Folder….
  - **735-05-here-panel (REQ-006):** a terminal "docs — sleep 600", "fake claude in …/repo/docs";
    the check: its log holds `repo/docs`.
  - **735-06-plus (REQ-007):** the project's `+`: New Terminal, New Browser Tab, New Agent…, New
    Agent Thread ›, then Agent CLIs.
  - Home's New Agent… button: the review of the diff (`home_page.rs`); Home is not shown in the run.
  - Focus report: one Marley window before and after on Chad's Hyprland, no rule added.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: New Agent asks where); the architecture note
  `docs/marley_architecture/marley_workbench.md` ("Where the agent starts", under #450's section);
  the slice line in `docs/marley/workbench-shell.md`; the guide came with Phase 2. The
  `project_panel.rs` row in `zed-touchpoints.md` describes what shipped.
- **Knowledge appended:** F-claude-735-the-picker-read-the-workspace-it-was-opened-in-001,
  F-claude-735-browse-opened-a-modal-from-inside-the-picker-001,
  PR-claude-735-code-in-a-workspaces-update-reads-it-by-reference-001.
- **Brain:** no `rusty` MCP server in this repository's sessions; no brain loop ran.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** `735-gate-2.log`, GATE GREEN [diff], 17 passed, on the tree committed.
