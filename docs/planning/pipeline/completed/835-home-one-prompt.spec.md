---
pipeline_id: a58e3793-1a30-4596-979e-5d8c370852d2
ticket: docs/planning/tickets/open/TICKET-835-home-one-prompt.md
status: Phase 4 — Complete PASS
title: Home: one prompt, then what needs you
type: feature
slice: The Marley look, W2.2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Home opens on one question: what should an agent do? A message box with the installed agents as
chips (one picked) and Start; typing a task and pressing Enter, or Start, runs the picked agent
with it in Home's project. Under the box, the START actions and Threads are a line of links; then
the cards as before (agents at work, recent projects, Rusty's tasks and follow-ups, agent
activity), and CONFIGURE's entries are links. Today Home opens on two cards of buttons, START and
NEW AGENT, and an agent starts empty.

## Scope
### In
- A PROMPT card first on Home: a single-line editor ("What should an agent do?"), the installed
  agent CLIs as chips with the picked one lit (the first installed by default), the folder the
  agent will start in, and Start. Start and Enter run
  `agents::start_cli_with_prompt(workspace, kind, task, …)` in Home's workspace and clear the box;
  an empty box starts the agent empty, as the old chip did.
- Under it, a line of links: New Terminal, Open Folder…, Clone Repository…, Command Palette, New
  Agent…, Threads (the old START buttons and NEW AGENT's two).
- The START and NEW AGENT cards removed.
- CONFIGURE's entries drawn as links (subtle buttons, accent text on hover).

### Out (explicitly deferred)
- A project chip to start the agent elsewhere: TICKET-867.
- Zed's agent (the Agent Panel) as a chip: the chips are the CLI agents, as NEW AGENT's were.

## Reference (§20)
N/A — Marley-specific. Home is Marley's own page (`home_page.rs`, #700); Zed's welcome page has no
agent prompt. The look is the design canvas's Home board.

### Prior art
- The code we ship: `MarleyHome::render_start`, `render_agents` (`agents::installed_clis`,
  `cli_icon`, `start_cli`, `show_picker`), `agents::start_cli_with_prompt` (#557, #826 start a
  CLI with a prompt), `terminal_view::default_working_directory`, `Editor::single_line` (the
  rail's filter) and the global `enter` → `menu::Confirm` binding for single-line editors.
- Ledger: #826's L-claude-826 (write a scenario with a fixture commit through the file tool),
  PR-claude-687 (check the stand-in runs before typing). The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/835-home-one-prompt.sh` (sway): project `repo`; stand-in `claude` and
`codex` first on PATH that print their arguments and wait; Home opened from the status bar.
- `835-01-night`: Home: the PROMPT card first with the box, the Claude Code and Codex chips
  (Claude Code lit), the folder, Start, the line of links; the other cards after; CONFIGURE as
  links (REQ-001, REQ-002, REQ-004).
- `835-02-started`: Codex picked, "write the README" typed, Enter: a terminal runs the stand-in
  `codex` with the task (REQ-003).
- `835-03-day`: Home in Marley Day (REQ-001, REQ-002, REQ-004).

## Locked-In Decisions
- D1 — The prompt starts a CLI agent through the same path as #826's Ask an agent
  (`start_cli_with_prompt`), in Home's own workspace (its group's project, or the home folder).
- D2 — The picked chip is the Home page's state for the session; it starts at the first installed
  CLI.
- D3 — Removing START and NEW AGENT keeps every action they had, as links or chips.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Home shall open with a prompt box, the installed agents as chips with one picked, and Start, in Marley Night and Marley Day. | `835-01-night`, `835-03-day` |
| REQ-002 | Home shall keep every START and NEW AGENT action as a link under the prompt. | `835-01-night` |
| REQ-003 | WHEN the user types a task and presses Enter or Start, Marley shall start the picked agent with that task in Home's project. | `835-02-started` |
| REQ-004 | Home's CONFIGURE entries shall draw as links. | `835-01-night`, `835-03-day` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the prompt card, the links, the configure links; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
