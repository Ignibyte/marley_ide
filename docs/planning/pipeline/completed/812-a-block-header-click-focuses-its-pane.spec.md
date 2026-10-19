---
pipeline_id: 191bd21d-dfa1-41e7-9b5f-1a6829760a95
ticket: docs/planning/tickets/closed/TICKET-812-a-restored-blocks-header-takes-the-panes-click.md
status: Phase 4 — Complete PASS
title: A click on a block header focuses its terminal
type: bug
slice: the block terminal (#628 native headers)
references: []
---

## Title
A press on a block's header (#628) stops there, so the terminal under it starts no selection and
its program gets no click. The press also never reaches the terminal element's own
`window.focus`, so in a split a click on the other pane's header leaves the focus where it was,
and what is typed next goes to the wrong terminal. The header should focus its terminal first.

## Scope
### In
- The header's left mouse-down (`block_headers.rs`, `header`) focuses the header's terminal view
  before it stops the press.
- The pinned command header's (#529, `sticky_header.rs`) left mouse-down, which stops the press
  the same way and jumps to the block's start, focuses its terminal view too (found at Phase 4).

### Out (explicitly deferred)
- Selection or a program's click over a header: unchanged (still none, by #628's design).
- A right-click on a header: the terminal's right-click menu is a separate path, untouched.

## Reference (§20)
Upstream Zed: `terminal_view::terminal_element` focuses the terminal's focus handle on a left
press (`window.focus(&focus, cx)` in its mouse-down listener) before it passes the press to the
terminal. The header keeps that first half of Zed's behavior and leaves out the second, as #628
decided.

### Prior art
- `crates/terminal_view/src/terminal_element.rs`: the left mouse-down listener calls
  `window.focus(&focus, cx)`, where `focus` is the `TerminalView`'s own handle
  (`terminal_view.rs`, `TerminalElement::new(.., self.focus_handle.clone(), ..)`).
- `gpui::Focusable` gives `Entity<TerminalView>` its `focus_handle(cx)`; `Window::focus(&handle,
  cx)` moves the focus, and Zed's pane follows a focused item.
- PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001: the other direction (a
  child that must not hand its click to a focusing parent); nothing on a child that has to focus.
- Nothing in Warp's maps applies: Warp's blocks are not drawn over a terminal grid.

## UI proof
`script/e2e/812-a-block-header-click-focuses-its-pane.sh`, under `compositor sway` (it clicks):
a terminal, a split with a second terminal, a command run in each so each pane draws a block
header.
- `812-01-left-focused`: a click in the left pane's body; the left terminal has the focus.
- `812-02-header-clicked`: a click on the right pane's block header, then a typed command; the
  command shows in the right pane, and the log the scenario reads names the right terminal
  (REQ-001, REQ-002).
- `812-03-pinned`: the right terminal scrolled back into `seq 1 400`, its command pinned over the
  top row.
- `812-04-pinned-clicked`: a click in the left pane, then on the right pane's pinned header, then
  `where`: the right pane jumped to the block's start and the command ran there (REQ-004).

## Locked-In Decisions
- D1 — The header focuses `view.focus_handle(cx)`, the same handle the terminal element focuses,
  so Marley's prompt editor (#627) and Zed's pane activation follow as they do for a click on the
  grid.
- D2 — It still stops the press and the release (#628): no selection, no click to the program.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks a block header in a pane that does not have the focus, Marley shall move the focus to that pane's terminal. | Shot `812-02-header-clicked`; the scenario's `expect` on its log |
| REQ-002 | WHEN the user types after that click, the typed command shall run in the clicked pane's terminal. | Shot `812-02-header-clicked`; the scenario's `expect` on its log |
| REQ-003 | WHEN the user presses on a block header, the terminal shall start no selection and report no click to its program. | The review of the diff (the stops stay) |
| REQ-004 | WHEN the user clicks a terminal's pinned command header (#529) in a pane that does not have the focus, Marley shall move the focus to that terminal and scroll it to the block's start. | Shot `812-04-pinned-clicked`; the scenario's `expect` on its log |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the header's mouse-down; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: the scenario on the debug build, every shot read.
- **P4 Complete** — CHANGELOG and the crate note (§21), ledger capture (§19), close the ticket,
  archive, commit.
