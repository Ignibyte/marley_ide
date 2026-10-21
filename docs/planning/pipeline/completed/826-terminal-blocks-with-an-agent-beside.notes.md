# Terminal blocks framed, timed, and asking an agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-826-terminal-blocks-with-an-agent-beside.md
- **Pipeline spec:** 826-terminal-blocks-with-an-agent-beside.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #825; Chad's overnight goal.
- **Classification / tier:** feature, L scoped to M: the side agent panel and the progress line are
  TICKET-865 (queued before #794).
- **Recall (§18.3):** L-claude-474 (buttons over the terminal stop the press; the chip's do);
  #557's English ask (`english::ask`, deferred, starts Claude Code with the line); the brain
  (consultation a98ebd84): nothing on this seam.
- **Discovery** (an Explore map, this session): `terminal_element.rs` spans 1576-1622, wash
  2058-2073, gutter bar 2121-2137, selected outline 2141-2164 (`gpui::outline` over
  `marley_rows_bounds`), `marley_bar_color` 2780, `marley_pill` 2795, `marley_block` 2853;
  `bookmarks::chip` (181), `send_block::chip` (136) / `send` (80) / `text_for` (29) / `targets`;
  `english::ask` (254); `agents::start_cli_with_prompt` (352); `agent_bar::footer_without_agent`
  (164), `render`'s chip closure (~216), `branch_of` (112); `duration_label`
  (`anchored.rs:130`).
- **Decisions:** D1 to D3 in the spec.

### Design
- **Frame** (`terminal_element.rs`, Zed): in the gutter loop, after the bar, paint
  `gpui::outline(rows, color.opacity(0.35), Solid)` over `marley_mapped_bounds(marley_rows_bounds(..))`
  for each span. Row 1xx (`terminal_element.rs`) gains the clause.
- **Duration** (`bookmarks.rs` / a new fn in `send_block.rs` or `blocks.rs`, Marley): in
  `bookmarks::chip`'s list, a muted XSmall label `duration_label(finished - started)` for a
  finished live block at `index`.
- **Ask and Explain** (`send_block.rs`, Marley): `enum Request { Bare, Fix, Explain }` with
  `words()` ("This command failed. Fix it:" / "Explain why this command failed. Change
  nothing:"); `send(view, index, request, window, cx)`; `send_now` prefixes the text; with no
  target it starts Claude Code through `workspace.update(|workspace, cx|
  agents::start_cli_with_prompt(workspace, AgentKind::Claude, &text, None, window, cx))`
  (detached), as `english::ask` does, in place of the toast. `chip` returns both buttons in an
  `h_flex` for the newest failed block at the prompt with no agent in its terminal (the targets
  check gone). `blocks.rs`' callers pass `Request::Bare`.
- **Prompt chips** (`agent_bar.rs`, Marley): the chip closure lifted to a fn `place_chip(icon,
  text)`; in `footer_without_agent`, when `prompt_editor` is some, a row of the folder
  (`terminal.working_directory()`, compacted) and branch (`branch_of`) chips under it, with
  `debug_selector`s.
- **File manifest:** `terminal_element.rs` (Zed), `send_block.rs`, `bookmarks.rs`, `blocks.rs`,
  `agent_bar.rs` (Marley), `docs/marley/zed-touchpoints.md`, the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001..004 | `true`, `sleep 2`, `false` typed at the prompt editor | `826-01-blocks` |
| REQ-003 | Explain clicked (point from the probe); a stand-in `claude` on PATH prints its args | `826-02-explain` |
| REQ-001/2/4 | back to the first terminal; Marley Day | `826-03-day` |

### Risks
- The frame's outline may collide with the gutter bar and the wash; the shot decides the alpha.
- Starting Claude Code from the chip runs in a new terminal: the stand-in must be first on the
  terminal's PATH (the scenario's `.bashrc`).

## Phase 2 — Code
- **Built:** `terminal_element.rs` (Zed): after each span's gutter bar, `gpui::outline` of the
  span's mapped rows in the bar's color at 35 %. `send_block.rs`: `Request { Bare, Fix, Explain }`
  with its words; `send(view, index, request, …)`; `send_now` prefixes the request and, with no
  target and a request, starts Claude Code through `agents::start_cli_with_prompt` (the bare
  block keeps its toast); `chip` offers Ask an agent and Explain on the newest failed live block at
  the prompt with no agent in its terminal (the targets check gone); `duration(terminal, index)`
  for a finished live block. `bookmarks::chip` adds the duration last. `blocks.rs` passes
  `Request::Bare`. `agent_bar.rs`: `place_chip` lifted out of `render`; `footer_without_agent`
  adds the folder and branch row under an open prompt editor.
- **Deviations:** none from the plan.
- **Review of the diff:** the chip's buttons stop propagation (L-claude-474); a 127 English ask
  still returns first, unchanged.
- **Gate:** `script/gates.sh --diff` 17/17 on the first run.

## Phase 3 — Test
- **Scenario:** `script/e2e/826-terminal-blocks-with-an-agent-beside.sh` (sway, the gated tree),
  exit 0, three shots. Two probe runs before the gate placed Explain and fixed a quoting note.
- `826-01-blocks`: `true` and `sleep 2` framed in green with `0 s` and `2 s` beside their check
  pills; `false` framed in red with Ask an agent, Explain, `0 s` and `exit 1`; under the prompt
  editor, the folder chip and `main` (REQ-001 to REQ-004 met).
- `826-02-explain`: Explain clicked with no agent running: a second terminal ran
  `claude 'Explain why this command failed. Change nothing: …'`, and the stand-in printed the
  request and the block (`$ false`, `exit 1 · 0 ms · <folder>`); its own block, running, framed in
  the running color (REQ-003 met).
- `826-03-day`: Marley Day, the first terminal: the frames, durations, chips and buttons in Day's
  colors (REQ-001, REQ-002, REQ-004 met).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #826); the guide's blocks section; the architecture note's blocks;
  look-plan TICKET-826 done with #865; the `terminal_element.rs` row checked.
- **Knowledge:** L-claude-826-a-hook-reads-the-whole-command-line-for-its-words-001.
- **Brain:** decision on consultation a98ebd84.
- **Ticket** closed; #865 filed; pipeline archived; committed on `marley/workbench-shell`.
