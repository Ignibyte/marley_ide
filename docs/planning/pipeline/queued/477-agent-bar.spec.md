---
pipeline_id: 2e424288-7e27-4718-8e1f-bff780439fcf
ticket: docs/planning/tickets/open/TICKET-477-agent-bar.md
status: QUEUED — Phase 1 Plan drafted; ready to promote
title: The agent bar, with the folder and branch
type: feature
slice: prong 1 T7a
references: [docs/marley/three-prong-plan.md]
---

## Title
A bar under a terminal running a CLI agent, with the agent's name at the left and its folder and
git branch at the right, the container for #478 to #481.

## Scope
### In
- **Zed's hook:** `terminal_view` gains a gpui global, `MarleyTerminalFooter`, holding a renderer
  that `TerminalView::render` calls for an optional element below the terminal. The view's root
  becomes a flex column and the footer keeps its own height (`flex_none`), so the grid gives up
  the bar's rows while it shows. Small and additive.
- **Marley's bar** (`marley_workbench::agent_bar`), registered at `init`: while the terminal's
  foreground process is a known CLI agent (`marley_agent::agent_kind_of` on
  `Terminal::foreground_process_command_name`, as the rail does), it renders the agent's name at
  the left and, at the right, the folder (`Terminal::working_directory`, the home directory as
  `~`) and its branch: the project's git store, from the repository whose work directory holds
  the folder, the longest match.
- No bar without an agent in the foreground.

### Out (explicitly deferred)
- The buttons (#478 to #481); chips to reorder or hide; the file explorer and
  `/remote-control` chips.
- A bar for plain shells (a setting, if Chad wants the folder and branch there too).
- A branch for a folder outside the project's repositories.

## Reference (§20)
- **Warp:** "When you launch a supported agent inside Warp, the agent toolbelt appears
  automatically", with chips on its left and right; the folder and branch sit at its right
  (https://docs.warp.dev/agents/cli-agents/claude-code/, and Chad's session on 2026-09-23). No
  Warp code.
- **Upstream Zed:** the title bar's branch (`title_bar.rs`, from `RepositorySnapshot::branch`)
  and the status bar's items; the terminal view's render.

### Prior art
- **Behavior maps:** `docs/warp_architecture/subsystems/04-agent-ai-mcp.md` covers Warp's own
  agent, not the third-party toolbelt; the docs above are the reference.
- **Published material:** Warp's docs, as above.
- **Code we already ship:** `marley_agent::agent_kind_of` (claude, codex, gemini, opencode) and
  the rail's use of `foreground_process_command_name` (`rail.rs`); `Terminal::working_directory`
  (the foreground process's directory, polled); `GitStore::repositories` and
  `RepositorySnapshot.branch`; gpui globals. `foreground_process_command_name` reads argv and
  gives nothing when argv[0] holds a `/`: a CLI started by name from PATH, as `claude` is, is
  recognized.

## UI proof
UI-AFFECTING: a bar under the terminal while an agent runs.
- **Driven tests** (`marley_workbench`): a terminal over a real PTY whose script `exec`s a
  program named `claude` (a link to `sleep` on a scratch PATH) shows the bar, with the folder
  and the project repository's branch; the same terminal before the exec shows none; the grid's
  line count drops while the bar shows.
- **Live drive:** `just shot` with a seed that starts a stand-in `claude` in a terminal: the bar
  under it.

## Locked-In Decisions
- D1 — The bar shows only while an agent runs, as Warp's toolbelt does. Chad was asked on
  2026-09-23 and did not choose; a setting can add it to plain shells.
- D2 — Zed gets a footer hook, not the bar; the bar is Marley's code in `marley_workbench`.
- D3 — The bar is a row of its own below the grid, not an overlay, so the agent's own footer
  (Claude Code's mode line) stays visible. The PTY loses the bar's rows while it shows.
- D4 — The branch comes from Zed's git store, which follows checkouts.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a terminal's foreground process is a known CLI agent, the terminal shall show the agent bar below its grid | driven |
| REQ-002 | WHILE no agent runs in the foreground, the terminal shall show no bar and keep every row | driven |
| REQ-003 | The bar shall show the agent's name at its left and its folder at its right, the home directory as `~` | driven |
| REQ-004 | WHERE the folder is inside a repository the project tracks, the bar shall show that repository's branch beside the folder | driven |
| REQ-005 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; promotion re-verifies the seams and asks the brain.
- **P2 Code** — the footer hook (ledger row first), the bar in `marley_workbench`.
- **P3 Test** — the driven tests, negative checks, a capture, the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
