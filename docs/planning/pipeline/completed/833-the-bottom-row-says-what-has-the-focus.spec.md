---
pipeline_id: 663926b1-7516-4b11-8b4c-1a60d87c3ce0
ticket: docs/planning/tickets/open/TICKET-833-the-bottom-row-says-what-has-the-focus.md
status: Phase 4 — Complete PASS
title: The bottom row says what has the focus
type: feature
slice: The Marley look, W1.4 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The bottom row starts with what you are in. For a file it shows the file's type and its path in
the project. For a terminal it shows the shell or program, its folder, the last command and how it
ended, whether a command runs now, its blocks and bookmarks, and the tmux sessions. For an agent
thread it shows the agent and what it is doing. Whenever an agent anywhere waits on you, the row
says so, with Allow and Open in the row and how many more wait. Today the row is Zed's icons, the
same in every tab, and a waiting agent shows only in the rail.

## Scope
### In
- `FocusRow`, a Marley `StatusItemView` added on the left after Debug in every workspace. It
  follows the active pane item (`set_active_pane_item`) and observes that item's entity, plus
  the rail and the tmux hosts, so it redraws as they change.
- The file case: a chip with the extension (`rs`, `md`, …, or a file icon) and the
  project-relative path.
- The terminal case: `>_` and the tab's title, the working folder relative to the project, the
  last finished block (`<command> · exit <n>`, red when non-zero), `running: <command>` or `no
  command running`, `<n> blocks` with `· <m> bookmarked` when any, and `tmux <n>` when Marley
  looks on tmux hosts with sessions.
- The agent-thread case (a thread tab, #776's `ThreadTab`): the agent's name and the thread's
  state (working, waiting, idle, failed).
- The needs-you case, over any focus: the oldest inbox entry (`<agent> in <project>: <ask>`) with
  Allow (when the entry offers it) and Open, and `+<n> more waiting`. Allow and Open do what the
  rail's inbox buttons do.
- The rail's inbox reads and actions become `pub(crate)` for it; `command_snapshot` is reused for
  the terminal's last block.

### Out (explicitly deferred)
- The panel buttons as one labeled group with dots: TICKET-862.
- The cards above the row (language servers, problems, panels, edit predictions) and the
  debugging row: TICKET-863.
- Hiding Zed's own left items (search, LSP, diagnostics stay; they already change with the focus
  where they are editor items).
- The Agent Panel's dock thread (only thread tabs in the center are a focus here).

## Reference (§20)
Upstream Zed (`workspace::StatusBar`, `StatusItemView`): the status bar calls
`set_active_pane_item` on each item when the active item changes, and Zed's own
`ActiveFileName`, `CursorPosition` and `ActiveBufferLanguage` show facts of the active editor that
way. Marley adds one item of the same kind, whose terminal and agent facts come from Marley's own
blocks and rail. The layout is the design canvas's BottomRow board.

### Prior art
- The code we ship: `workspace/src/status_bar.rs` (`StatusItemView`, `insert_item_after`,
  `position_of_item`); `workspace/src/active_file_name.rs` (path from `project_path` and the tab
  tooltip); `status_buttons.rs` (`HomeButton`, `DebugButton`, the `observe_new` registration);
  `marley_terminal::AnchoredBlocks` (`live`, block `state`, `exit_code`, `command`) through
  `Terminal::marley_anchored`; the rail's `command_snapshot` → `CommandSnapshot`;
  `bookmarks::is_bookmarked`; `marley_rail::working_directory_label`; the rail's inbox
  (`InboxEntry`, `InboxAnswer`, `open_inbox_entry`, `answer_inbox`) reached as `MarleyHome::rail()`
  reaches the rail; `thread_tab::ThreadTab` and `agents::thread_agent_name`; `TmuxHosts`.
- Ledger: PR-claude-831-a-view-a-workspace-hook-makes-reads-the-workspace-only-after-new-001
  (a status item is made inside the workspace's construction: read the workspace only later),
  F-claude-438-a (read state that `refresh` keeps), L-claude-831/832 (scenario points from a
  probe run; `tmux_hosts` set). The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/833-the-bottom-row-says-what-has-the-focus.sh` (sway): project `repo` with
`src/main.rs`; a terminal that ran `true`, then `false`; `marley.tmux_hosts` `[]`; a second
terminal running #542's stand-in Claude Code that waits on the user.
- `833-01-terminal`: repo's terminal: `>_ repo — bash · false · exit 1 · no command running ·
  2 blocks` (REQ-002); `833-02-running`: `sleep 30` typed: `running: sleep 30` (REQ-002).
- `833-03-waiting`: b handed over, its stand-in waiting: the row's waiting chip with Open
  (REQ-004).
- `833-04-file`: b's `src/lib.rs` open: `rs src/lib.rs` beside the chip (REQ-001).
- `833-05-opened`: Open clicked in the row: b's terminal, where the agent waits (REQ-005).
- `833-06-day`: Marley Day (REQ-002, REQ-004).
- The agent-thread case (REQ-003) needs a Zed agent thread tab with a running model; the scenario
  opens one only if the profile has an agent configured, else it is read in the diff (see notes).

## Locked-In Decisions
- D1 — One item, `FocusRow`, at the left after Debug; Zed's items stay where they are.
- D2 — Needs-you shows over every focus and takes the middle of the row. It is the rail's oldest
  inbox entry, so the row and the rail agree.
- D3 — The terminal's last block is the last *finished* live block; a running one shows as
  `running:`.
- D4 — The item reaches the rail through the window's `MultiWorkspace` sidebar, read in
  `render`, never in `new` (PR-claude-831).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a file editor is active, the bottom row shall start with the file's type and its project-relative path. | `833-03-file` |
| REQ-002 | WHILE a terminal is active, the bottom row shall show its title, folder, the last finished command and its exit code, whether a command runs, and its block and bookmark counts, in Marley Night and Marley Day. | `833-01-terminal`, `833-02-running`, `833-05-day` |
| REQ-003 | WHILE an agent thread tab is active, the bottom row shall show the agent's name and the thread's state. | the diff review; a shot when a thread can run |
| REQ-004 | WHILE an agent waits on the user, the bottom row shall show the request with Allow (when offered) and Open, and how many more wait. | `833-01-terminal` |
| REQ-005 | WHEN the user clicks Allow or Open in the row, Marley shall do what the rail's inbox does for that entry. | `833-04-allowed` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `focus_row.rs`, its registration, the rail accessors; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
