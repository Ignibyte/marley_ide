---
pipeline_id: fa2666d8-faf1-432e-a3a5-5b0f5ebf9540
ticket: docs/planning/tickets/open/TICKET-908-bottom-row-for-a-terminal.md
status: Phase 4 — Complete PASS
title: The bottom row for a terminal
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/B-rail-chrome.md, docs/marley/design-review-2026-10-23/A-core.md]
---

## Title
With a terminal in front, the bottom row says where the terminal is and what went wrong, as the
BottomRow board draws it: the folder, the branch and its changed count, the failed command as a
button that asks an agent to fix it, the blocks and bookmarks, and the terminal's size. The tmux
count says how many sessions are detached, and a click on it opens the rail's Sessions view, as
the TmuxSessions board draws it.

## Scope
### In
- **Folder at the root** (Marley crate, `focus_row.rs` `FocusRow::render_terminal`). The row
  always names the terminal's folder, the project's root included, in the block header's short
  form (`block_headers::folder_label`, made `pub(crate)`: `~`, then `…/parent/name` past two
  folders). Today `marley_rail::working_directory_label` returns `None` at the root, so the row
  names nothing there. That function stays as it is for the rail.
- **Branch and changed count** (Marley crate, `focus_row.rs`). After the folder, the branch of
  the repository that holds the folder and its changed-file count ("main · 1 changed"), in the
  theme's violet (`terminal_ansi_magenta`). The branch comes from `agent_bar::branch_of`. The
  count comes from that repository's `status_summary().count`, through a sibling of `branch_of` in
  `agent_bar.rs` that returns both values. With no changes, the branch shows alone. Outside a
  repository, nothing shows.
- **The failed block as a button** (Marley crate, `focus_row.rs`). When the terminal's newest
  live block finished with a non-zero exit, the row draws a red chip with a dot,
  "<command> · exit N · Ask an agent". A click calls `send_block::send` with `Request::Fix` for that
  block. This is the path the block header's own Ask an agent button takes (#826): the one agent
  in another terminal of the window, the picker when there are several, a new Claude Code when
  there is none. The red text it replaces goes.
- **Blocks and bookmarks** (Marley crate, `focus_row.rs`). "N blocks · M bookmarked" is already
  drawn (#833, from `MarleyBlockMarks`). It moves to the right group with "no command running",
  as the board orders the row. The words stay the same.
- **Terminal size** (Marley crate, `focus_row.rs`). The terminal's grid size, "132×38" (columns ×
  rows, in the buffer font), from `Terminal::last_content().terminal_bounds` (`num_columns`,
  `num_lines`). It is the last part of the focus row.
- **tmux count** (Marley crate, `focus_row.rs`; `rail.rs` for one visibility change). The row
  reads "tmux: N · M detached" on a teal chip (the tmux kind's tint from #905's helper), where M
  counts `TmuxSession`s with `attached == false`. With none detached, it reads "tmux: N". A click
  shows the rail's Sessions view (`Rail::show_sessions`, made `pub(crate)`). If the rail is
  shrunk to its strip, the click opens it first, as the strip's Sessions tile does
  (`toggle_rail_strip`). If the rail is closed, the click opens it
  (`MultiWorkspace::open_sidebar`). The chip has a tooltip, "Show the tmux sessions".
- **The `>_` chip** (Marley crate, `focus_row.rs` `chip`). The terminal's chip takes the shell
  kind's tint from #905's shared per-kind tint helper (`item_kind.rs`), so the row and the tabs agree.

### Out (explicitly deferred)
- Determinate build progress on a running command ("cargo build · 1781 of 1813") and the running
  chip's count. Nothing parses a command's progress yet (A-core TerminalSession, "Determinate
  build progress"). It needs its own follow-up ticket, which also feeds the rail row (#910).
- The agent terminal's own row: an agent chip, the model, the mode, "Turns (3)", the context
  meter (A-core AgentTerminal). That is new behaviour beyond a plain terminal and needs a
  follow-up ticket.
- The bottom row for a thread (#907) and for the other screens (#909).
- The sessions page's own gaps (column headers, state chips, Show or Attach, B-rail-chrome
  TmuxSessions). The click opens the rail's view, not the page, and the page is left as it is.
- Explain and Ask Marley next to the failed chip. They stay in the block header (#826, #865).
  The board draws one action in the row.

## Reference (§20)
- Warp, for the terminal's blocks: `docs/warp_architecture/subsystems/03-terminal-session-core.md`
  §6, "Blocks — per-command grids + shell-hook DCS metadata". Each block keeps its own state and
  exit code, and the prompt's precmd fields carry the folder and the git branch. Marley's row reads
  the same facts from `marley_terminal`'s anchored blocks (`BlockState`, `exit_code`), which are
  Marley's own model.
- Upstream Zed, for git: `project::git_store::Repository`'s `branch` and `status_summary()`, which
  Zed's title bar (`title_bar.rs:1047`) and `git_ui`'s repository selector already read for
  the same branch and changed count. The row reads them; it does not compute status itself.

### Prior art
- `focus_row.rs:105-196` `FocusRow::render_terminal`: the `>_` chip and title, the folder from
  `marley_rail::working_directory_label` (`marley_rail/src/marley_rail.rs:1998`, `None` when the
  folder is the project root, which is why `tour-01-start.png` shows none), the last command's
  text and exit as red text (`:165`), "running: <cmd>" or "no command running", "N blocks · M
  bookmarked" from `MarleyBlockMarks` (already built), and "tmux N" from `TmuxHosts` (`:190`),
  with no click.
- `block_headers.rs:234` `folder_label`: the block header's short folder ("~", "…/parent/name"),
  made short after F-claude-630. The row reuses it so the header and the row name a folder alike.
- `agent_bar.rs:112` `branch_of`: the branch of the innermost repository of the project's
  `GitStore` that holds a folder (#477, #630). Zed's `Repository::status_summary()`
  (`project/src/git_store.rs:6378`, a `GitSummary` with `count`) is what `title_bar.rs:1047` and
  `git_ui/src/repository_selector.rs:258` read for a changed count. No Marley code reads it yet:
  `changed` appears in no prompt or header file, which is why the Terminal board's "2 changed" is
  also missing at the prompt.
- `send_block.rs:103` `send` with `Request::Fix` ("This command failed. Fix it:"), behind the
  block header's Ask an agent (`send_block.rs:234`, #826). It picks the agent, never the block's
  own terminal, and starts Claude Code when no agent runs. #899's `send_selection::ask_to_fix`
  (`send_selection.rs:225`) asks about the line under an editor's caret. It needs an editor, so
  it does not fit a terminal's block. The block's own path, `send_block::send`, is the one to
  reuse. #910's failed shell row should reuse the same call.
- `rail.rs:9655` `command_snapshot`: the last live block's one-line text, `running` and
  `exit_code`. It skips an agent's own block (#551) and history from before a restart (#764).
  The row needs the block's index for `send`, which it reads from `marley_anchored().live().last()`.
- `terminal/src/terminal.rs:568` `Content::terminal_bounds` and `:898`/`:906` `num_lines`,
  `num_columns`: the grid's size in cells, kept current on every resize.
- `tmux_sessions.rs:48` `TmuxSession` (`attached: bool`, any client) in `TmuxHosts`'
  `Listing::Sessions`. The rail's Sessions view opens through `Rail::show_sessions`
  (`rail.rs:5429`), from the view switch (`rail.rs:5408`) and from the strip's Sessions tile
  (`rail_strip.rs:181`, which also calls `toggle_rail_strip`). The Sessions page opens through
  `sessions_page::OpenSessionsPage`. The board's "click opens Sessions" sits under "In the
  bottom row, in a terminal", beside the rail's view and the shrunk rail's tile.
- The BottomRow board ("In a terminal": `>_ bash`, `~/…/repo`, `main · 1 changed` in violet
  `#c4b5fd`, the red chip with a dot "false · exit 1 · Ask an agent", then at the right "no command
  running", "4 blocks · 1 bookmarked", `132×38` in mono). The TmuxSessions board ("tmux: 6 · 4
  detached" on teal `#13282a`/`#7dd3c0`, "click opens Sessions"). Marley's theme maps `#c4b5fd`
  to `terminal.ansi.magenta` (`assets/themes/marley/marley.json:90`).
- The behaviour maps: `docs/warp_architecture/subsystems/03-terminal-session-core.md` §6. No
  crate we build owns a status row for a terminal. Zed's terminal has no status item.

## UI proof
Scenario `script/e2e/908-bottom-row-for-a-terminal.sh` (sway, for the clicks). Setup: a scratch
git repository on `main` with one changed file, opened with `open_path`. The scenario's `.bashrc`
puts a stand-in `claude` first on the terminals' PATH (F-claude-903), and #899's stand-in idles
in a second terminal of the window, through Marley's plugin hook, so it is the window's one agent.
Two detached tmux sessions are started on the run's own tmux server (`TMUX_TMPDIR`, F-claude-880),
and a third is attached from a third terminal tab.
- `908-01-root`: the project's terminal in front at the root, after `false`: the `>_` chip, the
  folder, "main · 1 changed", the red "false · exit 1 · Ask an agent" chip, "no command running",
  the block count, the size and "tmux: 3 · 2 detached" (REQ-001, REQ-002, REQ-003, REQ-005,
  REQ-006, REQ-007).
- `908-02-asked`: the chip clicked: the stand-in's terminal holds "This command failed. Fix it:"
  and the block (REQ-004).
- `908-03-bookmarked`: a block bookmarked: "N blocks · 1 bookmarked" (REQ-005).
- `908-04-resized`: the window's pane split right: the size has fewer columns (REQ-006).
- `908-05-sessions`: the tmux chip clicked with the rail shrunk: the rail opens on its Sessions
  view (REQ-008).
- `908-06-day`: the row of `908-01-root` in Marley Day (REQ-001, REQ-002, REQ-003, REQ-007).

## Locked-In Decisions
- D1 — The failed chip sends through `send_block::send` with `Request::Fix`, the block header's
  own Ask an agent, so the row and the header ask the same agent the same way. #899's
  `ask_to_fix` is the editor's path and is not used here.
- D2 — The folder shows at the project root too, in the block header's short form
  (`folder_label`). The rail keeps `working_directory_label`, which hides the root on a row under
  its own project.
- D3 — "N changed" is the repository's `status_summary().count`, the number Zed's title bar and
  repository selector show, so all three agree.
- D4 — "detached" counts sessions with no client attached anywhere. The board's chip does not tell
  "attached here" from "attached elsewhere", so the row does not either.
- D5 — The tmux chip opens the rail's Sessions view, not the Sessions page, as the board's "click
  opens Sessions" and the strip's Sessions tile do. It opens the rail first when the rail is
  shrunk or closed.
- D6 — Colours come from the theme: violet for the branch (`terminal_ansi_magenta`, the board's
  `#c4b5fd` in Marley Night), the error colour for the failed chip, and the shell and tmux tints
  from #905's per-kind helper. Nothing is hard-coded.
- D7 — The row keeps PR-claude-833: its root is `flex_none`, and each long part (the title, the
  folder, the failed command) has a cap and truncates.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a terminal is in front, the bottom row shall name its folder in the block header's short form, including when the folder is the project's root. | `908-01-root`, `908-06-day` |
| REQ-002 | WHILE a terminal's folder is in a repository, the row shall show that repository's branch and, when files changed, "· N changed", in the theme's violet. | `908-01-root`, `908-06-day` |
| REQ-003 | WHEN the terminal's newest block finished with a non-zero exit, the row shall draw a red chip with a dot, "<command> · exit N · Ask an agent". | `908-01-root`, `908-06-day` |
| REQ-004 | WHEN that chip is clicked, the system shall send the block with "This command failed. Fix it:" to the window's agent, as the block header's Ask an agent does. | `908-02-asked` |
| REQ-005 | WHILE a terminal is in front, the row shall show its block count and, when some are bookmarked, "· N bookmarked", at the right after "no command running". | `908-01-root`, `908-03-bookmarked` |
| REQ-006 | WHILE a terminal is in front, the row shall show the terminal's size as columns × rows, and the size shall follow a resize. | `908-01-root`, `908-04-resized` |
| REQ-007 | WHILE tmux sessions are listed, the row shall show "tmux: N · M detached" on the tmux tint, or "tmux: N" when none is detached. | `908-01-root`, `908-06-day` |
| REQ-008 | WHEN the tmux chip is clicked, the system shall show the rail's Sessions view, opening the rail first if it is shrunk or closed. | `908-05-sessions` |
| REQ-009 | The change shall stay in the Marley crates, with no Zed crate hunk, and the row's root shall stay `flex_none` with each long part capped. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `focus_row.rs`'s terminal case as above; `folder_label` and `show_sessions` made
  `pub(crate)`; a branch-and-count sibling of `branch_of`; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
