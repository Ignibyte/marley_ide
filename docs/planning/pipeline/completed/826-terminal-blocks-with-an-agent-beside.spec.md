---
pipeline_id: 42dca8ac-d75b-4838-b12f-6fb3e3931eb2
ticket: docs/planning/tickets/open/TICKET-826-terminal-blocks-with-an-agent-beside.md
status: Phase 4 — Complete PASS
title: Terminal blocks framed, timed, and asking an agent
type: feature
slice: The Marley look, W2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Each command in a Marley terminal reads as a block: a thin frame in its state's color (running,
passed, failed) and, once it ends, how long it took beside its exit. A failed block offers "Ask an
agent", which sends it to fix, and "Explain", which asks why it failed and to change nothing;
with no agent running, either starts Claude Code with the block. At a shell prompt, the prompt
editor shows the folder and the branch as chips, as the agent bar does. Today a block has a
gutter bar and a faint wash, no time, and "Ask the agent" only when another terminal of the
window already runs one.

## Scope
### In
- The frame: a 1 px outline over each block's rows in its state color at 35 % (the same color as
  its gutter bar), a `// Marley:` hunk in `terminal_element.rs`; the selected and search-scope
  outlines still draw over it.
- The duration: a muted `4 m 12 s` beside a finished block's pill, from the block's times, through
  Marley's `MarleyBlockChip` hook (no Zed change).
- Ask an agent and Explain on the newest failed block at the prompt with no agent in its own
  terminal (`send_block::chip`): each sends the block's text with its request (fix it; explain
  it and change nothing) to the window's agent terminal (or the picker), or starts Claude Code
  with it when none runs. The 127 English ask (#557) stays as it is.
- Folder and branch chips at the shell prompt: in `agent_bar::footer_without_agent`, while the
  prompt editor is open, a row under it with the folder and the branch, the agent bar's chips.

### Out (explicitly deferred)
- The agent panel beside the terminal and the running block's progress line: TICKET-865.
- Rerun and Copy output stay on hover, as #474 has them.

## Reference (§20)
Warp's blocks (`docs/warp_architecture/subsystems/03-terminal-session-core.md` — the block list,
each command and its output as a
selectable unit with its exit status) as observed behavior: a frame per block and its time come
from that shape, reimplemented on Marley's own block spans (#470) and hooks. The look is the
design canvas's AgentTerminal board.

### Prior art
- The code we ship: `terminal_element.rs`'s Marley hunks (`marley_spans`, `marley_gutter_bounds`,
  `marley_rows_bounds`, `marley_bar_color`, the selected outline with `gpui::outline`, #554),
  `MarleyBlockChip` and `bookmarks::chip`, `send_block::{chip, send, text_for, targets}`,
  `english::ask` and `agents::start_cli_with_prompt` (a new Claude Code with a prompt),
  `marley_terminal::duration_label` and `marley_anchored().times(index)`, `agent_bar::branch_of`
  and the agent bar's chips, `rich_input::element`.
- Ledger: L-claude-474 (an occluding child ends its group's hover: the chip row's buttons stop
  propagation), #557's ask path (deferred out of the workspace's update), PR-claude-798 (no pane
  bounds read in render). The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/826-terminal-blocks-with-an-agent-beside.sh` (sway): project `repo`;
`marley.prompt_editor` at every prompt; `marley.tmux_hosts` `[]`; a stand-in `claude` on the PATH
that prints the prompt it got and waits.
- `826-01-blocks`: `true`, `sleep 2`, then `false` ran: three framed blocks (green, green, red),
  each with its duration, the failed one with Ask an agent and Explain; the prompt editor with the
  folder and branch chips (REQ-001 to REQ-004).
- `826-02-explain`: Explain clicked: a new terminal runs the stand-in `claude`, which printed the
  explain request and the block (REQ-003).
- `826-03-day`: Marley Day, the first terminal (REQ-001, REQ-002, REQ-004).

## Locked-In Decisions
- D1 — The frame uses the gutter bar's color at 35 %, so the three states read the same way in
  both places, and no new color token is added.
- D2 — Ask and Explain show on the newest failed block whether or not an agent runs elsewhere;
  with none, they start Claude Code, as #557's English ask does.
- D3 — `SendBlockToAgent` (ctrl-shift-enter) and the Block menu keep sending the bare block; only
  the chip's two buttons add a request.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Each block shall be framed in its state's color: running, passed or failed, in Marley Night and Marley Day. | `826-01-blocks`, `826-03-day` |
| REQ-002 | WHEN a block finishes, it shall show how long its command took beside its exit. | `826-01-blocks` |
| REQ-003 | WHILE the newest block failed at the prompt, it shall offer Ask an agent and Explain, which send it with their request to an agent, or start Claude Code with it when none runs. | `826-01-blocks`, `826-02-explain` |
| REQ-004 | WHILE the prompt editor is open at a shell prompt, it shall show the folder and the branch as chips. | `826-01-blocks` |
| REQ-005 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the frame hunk and its row; the duration chip; Ask and Explain; the prompt chips;
  a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
