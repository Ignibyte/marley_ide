---
pipeline_id: a42d7cfc-5d32-4bf2-962f-68bfc26a1dc2
ticket: docs/planning/tickets/open/TICKET-916-terminal-block-look.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Terminal blocks as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md, docs/warp_architecture/subsystems/03-terminal-session-core.md]
---

## Title
Each block in a Marley terminal reads as the Terminal board's card: a rounded frame with a gap to
the next, a state dot before the command, the output set in under the command, a neutral frame
for a passed block, a red frame and a tinted header band for a failed one, honey for a running
one. A failed block at the prompt offers Ask an agent, Explain and Ask Marley in a row under its
output; a running block's time ticks in its header; its progress line runs along its bottom edge.

## Scope
### In
- `crates/terminal_view/src/terminal_element.rs` (Zed crate, the Marley hunks):
  - The frame (#826's outline at :2137): a rounded border, the board's 12 px radius (no more than
    half the frame's height), in `border` for a passed block or one with no exit code, `error` for
    a failed one, the running colour while it runs. The comfortable density's half-row gap stays
    between frames; the compact density keeps them edge to edge.
  - The running colour moves from `info` to `warning` (the boards' honey) for the frame, the
    pill and the wash, so every running mark matches.
  - The wash (#470) covers only a failed or running block's header rows, as the board's band; the
    output rows stay on the terminal's background.
  - The gutter is two cells wide while native headers are on (one cell today), so the header's
    state dot sits in it and the command and the output start together at column 0, set in from
    the dot. The gutter bar is not drawn while native headers are on; it stays for the shell's own
    prompts (`marley.block_headers` off), which have no dot.
  - A row inserted under the newest block's output, above the live prompt, when a new hook
    `MarleyBlockFooter` gives an element for that block; the display-row map (#631) inserts it as
    it inserts a tall header's row.
  - `marley_block` (the element over a block's rows) draws the element a new hook
    `MarleyBlockProgress` gives for a running block at its bottom edge.
- `crates/terminal_view/src/terminal_view.rs` (Zed crate): the two hook globals,
  `MarleyBlockFooter` and `MarleyBlockProgress`, beside `MarleyBlockChip`.
- `block_headers.rs` (Marley):
  - `header`: an 8 px dot in the state colour at the gutter, before the command's row.
  - `progress_line` moves to `MarleyBlockProgress`: 3 px along the bottom edge in the running
    colour, still the indeterminate slide (#865 D3).
  - A once-a-second notify of each terminal view while one of its live blocks runs, so its time
    ticks; nothing runs while no block does.
- `send_block.rs` (Marley):
  - `footer`, the `MarleyBlockFooter` hook: for the newest failed block at the prompt with no
    agent in front, a row of labelled buttons: Ask an agent (filled accent), Explain and Ask
    Marley (outlined). They do what #826's and #865's chip buttons do. The header's chip no
    longer carries them; the 127 "Ask the agent" chip (#557) stays in the header.
  - `duration`: a running block shows "running · <elapsed>" in the running colour, where the
    "running" pill was; a finished block keeps its duration.

### Out (explicitly deferred)
- Rerun and Copy output as labelled buttons in the action row: #826 ("Rerun and Copy output stay
  on hover").
- "Ask an agent to fix it" as the button's words: the board's wording; the shorter "Ask an agent"
  stays, as the ticket asks.
- A determinate progress bar ("1781 of 1813"), the block map and the exit-127 suggestion with the
  distro's command: TICKET-918. Whichever of 916 and 918 lands second moves 918's filled line to
  the bottom edge, and 918's distro hint into the footer row.
- The "N changed" chip: it sits at the prompt on the boards, so TICKET-917 (the prompt card)
  carries it.
- The start time in a header ("14:14 · 0.04 s") and the selected block's labelled toolbar
  ("selected · bookmarked", Copy … Send to agent): the TerminalSession board's remaining items, a
  follow-up ticket.
- The agent column inside the terminal: #865's Out.
- The bottom row's failed chip and branch: TICKET-908.

## Reference (§20)
Warp's blocks (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6, "Blocks —
per-command grids + shell-hook DCS metadata": each command and its output a unit with its state
and exit code) as observed behaviour: a block reads as one card with its state, its time and its
actions. Reimplemented on Marley's own block spans (#470), headers (#628) and row map (#631); no
Warp source read. The look is the Terminal and TerminalSession boards'.

### Prior art
- `terminal_element.rs:2121-2155`: the gutter bar (`marley_gutter_bounds`, :2688) and #826's frame
  (`gpui::outline` at 35 % of the bar's colour, no radius); `:2058` the wash (`marley_wash`,
  :2786); `:2796` `marley_bar_color` (running = `info`); `:2811` `marley_pill`.
- `terminal_element.rs:1331-1360`: the gutter is one cell (`gutter = cell_width`) taken from the
  grid's width.
- `terminal_element.rs:1576-1660`: the header hook, `MarleyBlockSpacing`, and
  `marley_terminal::RowMap::new(starts, tall, gap, line_height, anchor)`: `tall` already inserts
  one row above a block's start, and the live prompt is a start
  (`marley_live_prompt_row`), so a row above it is the same mechanism.
- `terminal_element.rs:2866-2988` `marley_block`: the element over a block's rows, with the pill,
  the hover actions and the chip on its first row.
- `terminal_view.rs:201-263`: `MarleyBlockChip`, `MarleyBlockBody`, `MarleyBlockHeader`,
  `MarleyBlockSpacing`, the pattern for the two new hooks.
- `block_headers.rs:128-200` `header` (place over command on two rows); `:204` `progress_line`
  (#865 D3).
- `send_block.rs:185-270` `chip` (Ask an agent, Explain, Ask Marley, and #557's 127 chip); `:304`
  `duration` (finished blocks only).
- `marley_terminal::duration_label` ("1 m 20 s") and `marley_anchored().times(index)` (a block's
  start).
- Theme: Marley Night `info` is #ffb27a (Marmalade), `warning` #f6c453 (the boards' honey).
- Ledger: L-claude-803-a-blocks-chip-moves-with-its-output-001 (measure a chip's point from the
  line count), PR-claude-812-an-element-that-stops-a-press-over-a-view-focuses-it-001 (the footer
  row stops presses, so it focuses the terminal itself), L-claude-474 (an occluding child ends
  its group's hover).

## UI proof
Scenario `script/e2e/916-terminal-block-look.sh` (sway): project `repo`; `marley.prompt_editor`
on; `marley.tmux_hosts` `[]`; a stand-in `claude` first on the terminals' PATH through the
scenario's `.bashrc`, which prints the prompt it got and waits.
- `916-01-blocks`: `git status --short`, then `false`: two rounded frames with a gap, the passed
  one neutral with a green dot, the failed one red with its header band and the action row under
  its output (REQ-001, REQ-002, REQ-003).
- `916-02-running`: `sleep 8` running: a honey frame and dot, "running · N s", the progress line
  at the bottom (REQ-004, REQ-005).
- `916-03-ticked`: two seconds later: the time moved on (REQ-004).
- `916-04-ask`: after `sleep` ends and `false` again, Ask an agent clicked: a new terminal runs
  the stand-in, which printed the fix request and the block (REQ-003).
- `916-05-day`: Marley Day, the blocks of `916-01-blocks` and a running one (REQ-001 to REQ-005).

## Locked-In Decisions
- D1 — Running is honey (`warning`) everywhere a block shows it, as every board draws a working
  state; `info` stays for what is not a state.
- D2 — The output is not moved off the grid's columns: the gutter widens and the dot sits in it,
  so the command and the output start at column 0 and the PTY's columns change by one cell, not
  by a pixel shift that would break selection.
- D3 — The action row is an inserted display row, so it pushes nothing out of the grid and goes
  when the next command starts.
- D4 — The ticking refresh runs only while a block runs, once a second per terminal view.
- D5 — The progress line stays indeterminate until a command's own progress is read (the
  follow-up).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Each block shall be drawn in a rounded frame with a gap to the next block and a state dot before its command, with its output starting under the command, in Marley Night and Marley Day. | `916-01-blocks`, `916-05-day` |
| REQ-002 | A passed block's frame shall be neutral; a failed block's frame shall be red with its header band tinted. | `916-01-blocks`, `916-05-day` |
| REQ-003 | WHILE the newest block failed at the prompt, a row under its output shall offer Ask an agent, Explain and Ask Marley, labelled, and each shall act as before. | `916-01-blocks`, `916-04-ask` |
| REQ-004 | WHILE a block runs, its header shall show "running ·" and its elapsed time, which advances each second. | `916-02-running`, `916-03-ticked` |
| REQ-005 | WHILE a block runs, its frame and dot shall be honey and its progress line shall run along its bottom edge. | `916-02-running`, `916-05-day` |
| REQ-006 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the frame, gutter, dot, band, footer row and progress hooks, the ticking time,
  the ledger rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
