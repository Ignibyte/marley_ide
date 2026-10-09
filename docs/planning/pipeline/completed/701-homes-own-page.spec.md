---
pipeline_id: b71d79c7-74bb-43e4-ab06-1f6be81ff3dd
ticket: docs/planning/tickets/open/TICKET-701-homes-own-page.md
status: Phase 4 — Complete PASS
title: Home's own page in place of Zed's Welcome page
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), on Chad's 2026-10-09 ask
references:
  - docs/planning/pipeline/completed/679-the-rusty-home-page.spec.md
  - docs/planning/pipeline/completed/699-the-rusty-groups-plus-menu-and-home-page.spec.md
  - docs/planning/pipeline/completed/700-home-and-rusty-groups-from-the-start.spec.md
---

## Title
The Home group gets its own page, a home page for Zed and Marley, in place of Zed's Welcome page.

Chad, 2026-10-09: "Home also has its own page instead of the open project. Come up with useful
things that are in zed that would be good. Opening a terminal, agents. A home page for zed +
marley basically."

## Scope
### In
- `MarleyHome`, a tab titled Home, first in the Home group, with cards:
  - **Start:** New Terminal (in Home, the home folder), Open Folder…, Clone Repository…, Command
    Palette;
  - **New Agent:** a button per agent CLI on the PATH (Claude Code, Codex, Gemini CLI, OpenCode),
    each started in a terminal in Home's home folder. Changed in Test from New Agent Thread: Zed's
    Agent Panel runs no thread in a folderless workspace (see the notes);
  - **Recent Projects:** Zed's recent local projects, each reopening on a click;
  - **Agents at Work:** the window's agent threads and agent CLIs that are working, waiting or
    failed, from the rail's snapshot, each opening where it lives;
  - **Configure:** Settings, Marley Settings, Keymap, Extensions, the Marley guide.
- The Home group shows its page: added first when the group is shown or emptied and has none, so
  an empty Home never shows Zed's Welcome page.

### Out (explicitly deferred)
- The page across a restart (`SerializableItem`); it is added again when Home is shown.
- A + menu of Home's own (#699 gave Rusty one); Chad did not ask for it.

## Reference (§20)
Upstream Zed, workspace: `welcome::WelcomePage` (Get Started: New File, Open Project, Clone
Repository, Command Palette; Configure: Settings, Keymap, Extensions; Recent Projects). Marley's
page keeps its useful entries and their actions, adds Marley's own (terminal, agents, the rail's
working agents, the Marley guide and settings), and is drawn as the Rusty home page is (#679).

### Prior art
- **The code we ship:** `welcome::WelcomePage` and its actions (`Open`, `git::Clone`,
  `command_palette::Toggle`, `OpenSettings`, `OpenKeymap`, `Extensions`);
  `WorkspaceDb::recent_project_workspaces` and `Workspace::open_workspace_for_paths` (its recent
  projects); `OpenSettingsAt` (the Marley page, as `agent_versions.rs` uses it).
- **Marley:** `rusty::home_tab` (cards, `ensure`, #699's `fill`); `agents::{thread_agents,
  start_thread, start_cli, launcher}` and `NewAgent` (the rail's + menu); `TerminalPanel::
  add_center_terminal` with `default_working_directory` (the rail's `new_terminal`); the rail's
  `RailSnapshot` and `open_row`; `OpenGuide`.
- **Behavior maps:** Warp's welcome tab is not mapped in `docs/warp_architecture/`; nothing taken.

## UI proof
`script/e2e/701-homes-own-page.sh`, under `compositor sway`, on a fresh profile with a stand-in
`claude` first on the terminals' PATH.

Shots:
- `701-01-home-page`: the Home group shows its page with its five cards.
- `701-02-recent`: a recent project clicked on the page: it opens in the rail.
- `701-03-terminal`: New Terminal: a terminal tab in Home.
- `701-04-agent`: the Claude Code button: a stand-in `claude` in a terminal in Home.
- `701-05-at-work`: the page's Agents at Work lists that terminal, working.
- `701-06-emptied`: every tab of Home closed: the page again, not Zed's Welcome page.

## Locked-In Decisions
- **D1:** the page is a Marley `Item` in a new module, `home_page.rs`, drawn as `RustyHome` is.
- **D2:** the page is ensured, first and without taking the active tab from another, whenever the
  Home group is shown or loses its last tab; #699's subscriptions serve both groups.
- **D3:** Start and Configure dispatch Zed's and Marley's actions from the page's focus handle;
  New Terminal and New Agent call the rail's helpers on the page's own workspace, from plain
  click closures, never `cx.listener`.
- **D4:** Agents at Work reads the window's rail (`MultiWorkspace::sidebar` downcast to `Rail`)
  and opens a row through the rail; in the Zed layout the card says the rail lists them.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the Home group is shown without its page, the system shall add Home's page first and show it when the group has no other tab. | Shot 701-01 |
| REQ-002 | WHEN the user clicks a recent project on Home's page, the system shall open that project. | Shot 701-02 |
| REQ-003 | WHEN the user clicks New Terminal on Home's page, the system shall open a terminal in the Home group. | Shot 701-03 |
| REQ-004 | WHEN the user clicks an agent CLI under New Agent, the system shall start it in a terminal in the Home group. | Shot 701-04 |
| REQ-005 | WHILE an agent thread or agent CLI in the window is working, waiting or failed, Home's page shall list it under Agents at Work. | Shot 701-05 |
| REQ-006 | WHEN the Home group's last tab closes, the system shall show Home's page, not Zed's Welcome page. | Shot 701-06 |
| REQ-007 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `home_page.rs`, `rail.rs` (the snapshot's agents at work, an opener), the shared
  fill in `rusty/home_tab.rs`; a review of the diff; the gate.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, architecture docs, ledger, close, archive, commit.
