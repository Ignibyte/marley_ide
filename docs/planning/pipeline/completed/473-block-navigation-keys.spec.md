---
pipeline_id: 0a7071ab-a5ea-4c77-8412-1396a42b2e4f
ticket: docs/planning/tickets/closed/TICKET-473-block-navigation-keys.md
status: Phase 4 — Complete PASS
title: Block navigation keys
type: feature
slice: prong 1 T1c
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/470-blocks-drawn-in-the-terminal.spec.md]
---

## Title
Stage one draws blocks (#470); this adds the plan's T1 navigation keys: scroll the focused
terminal to the start of the previous or the next block.

## Scope
### In
- **Where to scroll** (`marley_terminal::block_scroll`, pure): from a terminal's blocks, its
  screen's top as an absolute line, its scroll offset and its history, the scroll offset that
  puts the first line of the last block starting above the viewport's top, or of the first
  block starting below it, at the top; 0 when that line is on the live screen; within the
  history; `None` when there is no block that way.
- **The actions** (`marley::PreviousBlock`, `marley::NextBlock`, `marley_workbench`): caught at
  the workspace's root, as `routing` catches its actions; they act on the terminal view that
  holds focus, in the center or in the Terminal Panel, and scroll it through
  `Terminal::scroll_up_by` and `scroll_down_by`.
- **The keys** (the Marley keymap): `secondary-up` and `secondary-down` in `Terminal`.

### Out (explicitly deferred)
- A selected block, highlighted, as Warp's keys select one: stage one keeps no selection.
- Hover copy and rerun (T1b).

## Reference (§20)
- **Warp:** block navigation from the keyboard (`docs/warp_architecture/subsystems/03-terminal-
  session-core.md` §6, blocks as the unit of output), with Ctrl-Up and Ctrl-Down on Linux and
  Cmd-Up and Cmd-Down on macOS, as observed. Marley scrolls to a block's start; no Warp code.
- **Upstream Zed:** `Terminal::scroll_up_by` and `scroll_down_by`, and the `Terminal` key
  context, whose defaults leave `secondary-up` and `secondary-down` unbound.

### Prior art
- **Behavior maps:** as above.
- **Published material:** none needed.
- **Code we already ship.** `routing::init` catches actions at the workspace's root with
  `register_action_renderer`; #470's `Content::marley_screen_top` and `Terminal::blocks()`;
  `default-linux.json`'s `Terminal` context binds `shift-up`, `shift-down` and the page keys,
  not `ctrl-up` or `ctrl-down`.

## UI proof
UI-AFFECTING: keys that scroll the terminal.
- **Driven tests** (`marley_workbench`): a real PTY's terminal with scrollback and three blocks,
  focused in the workspace: `PreviousBlock` and `NextBlock` move its scroll offset to each
  block's start in turn and to the live screen; with no block that way it stays; the Marley
  keymap binds the two keys in `Terminal`.
- **Live drive:** needs keys, and Chad is at the desk; recorded, not skipped: the driven tests
  press nothing either, and dispatch the actions the keys are bound to.

## Locked-In Decisions
- D1 — The keys scroll; they select nothing (stage one has no block selection).
- D2 — The actions live in `marley_workbench` and change no Zed path.
- D3 — "Previous" and "next" are counted from the viewport's top line.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `PreviousBlock` runs in a focused terminal, it shall scroll the start of the last block beginning above the viewport's top to the top, within the history | unit; driven |
| REQ-002 | WHEN `NextBlock` runs, it shall scroll the start of the first block beginning below the viewport's top to the top, or to the live screen when that start is on it | unit; driven |
| REQ-003 | WHEN no block begins that way, the terminal shall stay where it is | unit; driven |
| REQ-004 | The Marley keymap shall bind `secondary-up` and `secondary-down` to them in `Terminal` | driven |
| REQ-005 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec. **P2 Code** — `block_scroll`, the actions and handlers, the keys.
  **P3 Test** — the tests, negative checks, the gate. **P4 Complete** — docs, ledger, close,
  archive, commit.
