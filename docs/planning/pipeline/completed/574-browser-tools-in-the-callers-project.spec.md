---
pipeline_id: 21b825a4-b711-4534-9240-b2c3047fe371
ticket: docs/planning/tickets/closed/TICKET-574-browser-tools-in-the-callers-project.md
status: Phase 4 — Complete PASS
title: "Browser tools act in the caller's project"
type: feature
slice: prong 3 with prong 2; #520's second piece (the browser tools read #520's caller)
references: [docs/planning/pipeline/completed/520-terminal-identity.spec.md, docs/planning/pipeline/completed/520-terminal-identity.notes.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/orca_architecture/06-cli-automations-skills.md]
---

## Title
#520 tells Marley's tools which terminal, project and folder a call comes from. The browser tools
still act on the tab the user focused last, whatever its project, so an agent working in one
project can drive another project's page. Here a browser tool that names no tab acts in the
caller's project: on the tab the user focused last there, and `browser_navigate` opens a new tab
there when the project has none. `browser_tabs` says each tab's project and which tab a call with
no tab would act on. A caller with no project keeps today's behavior.

## Scope
### In
- **The caller's project** (`crates/marley_workbench/src/browser_tools.rs`): the project group of
  the caller's terminal when `Caller::terminal` names one of Marley's terminals; else the group
  one of whose workspaces' folders holds `Caller::project`; else the one holding `Caller::cwd`,
  the longest folder winning. Groups come from each window's `MultiWorkspace::project_groups`;
  folders are each workspace's own (`Workspace::root_paths`), not the group's key, which holds
  main worktree paths only, so a linked worktree (#510) finds its group.
- **The default tab.** The hub keeps its focus history (most recent last) instead of the one
  page the user focused last. A tool that names no `tab`, for a caller with a project, acts on
  the newest page in that history, still open, whose Browser tab belongs to one of the project's
  workspaces. With none, `browser_navigate` opens a new tab in the project (the caller's own
  workspace, placed as AD-claude-493 places an agent's page: behind the tab in front, or in a
  split beside the pane with the focus when that pane shows other work), and every other tool
  refuses with its next step (`browser_navigate` opens one in this project).
- **`browser_tabs`** gives each tab's `project` (its group's name) and marks with `default: true`
  the tab a call with no `tab` would act on for this caller; `focused` keeps its meaning (the tab
  the user focused last, anywhere).
- **Zed's own agents.** An Agent Panel agent's bridge runs in its project's root (Zed starts a
  local project's context servers there), with #520's variables blank: its `Marley-Cwd` places it.
- The browser tools' descriptions say what a call with no `tab` acts on.

### Out (explicitly deferred)
- #507's browser context per project (cookies and logins apart): this ticket only chooses the tab.
- Refusing a named `tab` of another project: a named tab is the caller's explicit choice.
- The id surviving a restore (#575).

## Reference (§20)
- **Orca:** browser commands default to the active tab of the caller's worktree, the worktree
  whose path holds the caller's working directory (report 03 §2.11 and item 4,
  `getBrowserCommandTarget` in `src/cli/selectors.ts`; report 06 §2.5's cwd rule). Marley does the
  same with #520's caller, by project group.
- **Upstream Zed:** project groups and their workspaces (`MultiWorkspace::project_groups`,
  `Workspace::root_paths`), and context servers started in the project's folder
  (`crates/project/src/context_server_store.rs`), kept as they are.
- **Warp:** N/A. Plumbing between an agent and Marley's browser tools, no Warp behavior to match.

### Prior art
- **Behavior maps.** Orca's reports above. AD-claude-493-one-browser-tab-per-page-001: one tab per
  page; an agent's page never takes the focus and opens behind the tab in front, or in a split
  beside a pane showing other work; the tools act on the named tab or the tab focused last. This
  ticket keeps the placement rules and changes only the default and the workspace.
- **Published material.** MCP's Streamable HTTP (the bridge's headers ride its requests).
- **The code we already ship.** #520's `marley_mcp::Caller` on each `AppCall` and
  `Terminal::marley_terminal_id`; `mcp::terminals` (every terminal with its workspace);
  `BrowserHub::focused` and `set_focused` (`browser.rs:520`, `:527`, the one value this replaces
  with a history), `tabs` (`:535`); `place_tab` (`:2798`: beside the opener, else after the tab
  focused last, else the active workspace) and `BrowserView::workspace` (`:3005`);
  `browser_tools::page_of` (`browser_tools.rs:115`) and `navigate` (`:432`);
  `MultiWorkspace::project_groups` (`multi_workspace.rs:849`), `Workspace::root_paths`
  (`workspace.rs:7685`). Does a crate we build own this seam? No: Zed has no browser; the groups
  and folders are Zed's, the choice is Marley's.

## UI proof
UI-AFFECTING: which Browser tab an agent's call drives.
`script/e2e/574-browser-tools-in-the-callers-project.sh` (`compositor sway`; an offline Chromium
serving two local pages, `browser-fixture.sh`). Two projects, A and B, in Marley's window (the
second opened with #513's handoff, a second `marley B`). Shots:
- `574-01-two-projects`: A and B in the rail, B with a Browser tab on page one, the tab the user
  focused last.
- `574-02-scoped-navigate`: in A's terminal, the stand-in agent's `navigate <page two>` with no
  tab: a new tab opens in A, and B's tab still shows page one.
- `574-03-tabs`: the stand-in's `tabs` from A: each tab's project, A's new tab marked `default`,
  B's tab still `focused`.
- `574-04-cwd`: the stand-in run from B's folder with no terminal id (as Zed's agents' bridge
  runs) navigates with no tab: B's tab goes to page two.
The harness-side client (no terminal, no project, a folder outside both) navigates with no tab:
the tab the user focused last moves, as before this ticket (the run log).

## Locked-In Decisions
- D1: The caller's project comes from its terminal, else `Marley-Project`, else `Marley-Cwd`,
  matched against each group's workspaces' own folders, the longest folder winning (#520's D6).
- D2: A caller with no project keeps today's behavior: the tab the user focused last, anywhere
  (#520's D7).
- D3: A caller with a project never acts on another project's tab by default: with no tab of its
  own, `browser_navigate` opens one and the other tools refuse with the next step. Driving a page
  the user did not open for this project is the failure this ticket exists to stop.
- D4: The hub keeps its focus history rather than one value, filtered to live pages as the one
  value was, so "the newest tab of project P the user focused" needs no second record.
- D5: A new tab for a caller's project goes to the caller's own workspace (its terminal's, or the
  one whose folder held its path), through a placement the hub keeps for the new page's id and
  consumes when the page's tab opens, and keeps AD-claude-493's rules. (Phase 2: the caller's
  workspace rather than the group's last active one, so a linked worktree's agent gets its tab
  beside its own terminal; keyed by the page rather than one-shot, so a page opened meanwhile
  cannot take it.)

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a browser tool that names no tab comes from a terminal of project P, it shall act on the tab of P the user focused last, whichever tab the user focused last overall. | The run log: a `look` from A reads A's tab after B's was focused |
| REQ-002 | WHEN `browser_navigate` names no tab and P has no Browser tab, the system shall open a new tab in P and leave other projects' tabs as they are. | Shot `574-02-scoped-navigate` |
| REQ-003 | WHEN a tool other than `browser_navigate` names no tab and P has no Browser tab, it shall refuse, naming `browser_navigate` as the next step. | The run log |
| REQ-004 | WHEN `browser_tabs` is called, each tab shall carry its `project`, and the tab a call with no tab would act on for this caller shall be `default`. | Shot `574-03-tabs`; the run log |
| REQ-005 | WHEN a caller has no terminal id and its folder is inside P, as Zed's agents' bridge is, the tools shall act in P. | Shot `574-04-cwd` |
| REQ-006 | WHEN a caller has no terminal and no project, the tools shall act on the tab the user focused last, as before. | The run log: the harness-side client |
| REQ-007 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec and the design in the notes.
- **P2 Code:** the hub's focus history, `focused_among`, the one-shot placement; the caller's
  project; `page_of`, `navigate` and `tabs`; the descriptions. fmt and clippy clean; a review.
- **P3 Test:** write and run the scenario, read every shot, `just regress`, `script/gates.sh --diff`.
- **P4 Complete:** CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the ledger, close,
  archive, commit.
