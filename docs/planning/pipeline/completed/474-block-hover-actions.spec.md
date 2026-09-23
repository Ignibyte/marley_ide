---
pipeline_id: 57cc60db-73e2-46cd-8627-f51ba7332cd5
ticket: docs/planning/tickets/open/TICKET-474-block-hover-actions.md
status: Phase 4 — Complete PASS
title: Copy and rerun on a hovered block
type: feature
slice: prong 1 T1b
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/470-blocks-drawn-in-the-terminal.spec.md]
---

## Title
Stage one's hover actions (the plan's D3, T1): Copy and Rerun on the block under the pointer.

## Scope
### In
- **A block's element** (`terminal_view::TerminalElement`): each block whose first row is on
  screen gets one element over its rows, with a hover group; its first row holds the pill
  (#470) and, visible while the group is hovered, the two buttons; a press on one does not
  reach the terminal.
- **Copy:** the block's output (`Terminal::block_output`) to the clipboard; nothing when its
  lines have left the scrollback.
- **Rerun:** offered only while no block runs (the shell is at its prompt) and the command is
  not empty; it sends Ctrl-U, the command and a carriage return, so a half-typed line does not
  prefix it.
- **The shell's own commands** (added in Code, D4): each local terminal gives its program a
  random nonce in `MARLEY_SHELL_NONCE`; Marley's bash and zsh integrations take it out of the
  environment before anything else runs and add it to each `preexec` frame; a block whose
  `preexec` carried it records its command as verified, and only such a block is offered Rerun.
  Output that prints a frame of its own cannot know the nonce.

### Out (explicitly deferred)
- Actions for a block whose first row is scrolled away.
- A menu of more actions (copy the command, share, bookmark).

## Reference (§20)
- **Warp:** a block's hover actions, observed: copy output and rerun among them
  (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6). No Warp code.
- **Upstream Zed:** `TerminalView`'s rerun button for tasks (`IconName::Rerun`), the icons and
  `ui::IconButton`, and gpui's group hover.
- **VS Code's shell integration** (published): the command line its script reports
  (`OSC 633 ; E`) carries a nonce the terminal gave the shell in `VSCODE_NONCE`, so the terminal
  can tell the script's reports from output that imitates them.

### Prior art
- **Behavior maps:** as above.
- **Published material:** readline's `unix-line-discard` on Ctrl-U (bash, and zsh's emacs
  keymap).
- **Code we already ship.** #470's pill element and its placement; `Terminal::block_output`
  (#464) and `Terminal::input`; gpui's `should_insert_hitbox` gives a `.group()` element a hitbox,
  and `visible_on_hover` repaints on a hover change. `ui::ButtonLike` stops its click's release
  from propagating but not the press (`button_like.rs:872-876`).

## UI proof
UI-AFFECTING: buttons over the terminal and what they send.
- **Driven tests** (`terminal_view`): over a real PTY's blocks, with the pointer moved onto the
  failed block, Copy puts its output on the clipboard and Rerun sends `\x15false\r`; with mouse
  reporting on, a press on a button sends the program nothing, while a press on the output and
  a release over a button are reported; while a block runs no Rerun is drawn, nor for a command
  whose frame did not carry the terminal's nonce.
- **PTY tests** (`terminal`): a real bash and a real zsh send the nonce in their command frames
  and leave it out of their children's environment; a remote terminal's program is given none.
- **Live drive:** a capture shows no button without a pointer; hovering needs input, recorded,
  not skipped.

## Locked-In Decisions
- D1 — One element per block holds the pill and the buttons, so the buttons can show on the
  whole block's hover (gpui's groups reach descendants only).
- D2 — Rerun only at a prompt, with Ctrl-U first.
- D3 — A press on a button stays off the terminal; the rest of the block's element takes no
  mouse event, so selection and clicks in the terminal work as before. Revised in Code: the
  plan had the buttons occlude, but an occluding hitbox ends the block's hover under it, so a
  wrapper stops the press instead (the notes' Phase 2).
- D4 — Rerun sends only a command the shell's own hook reported: the `preexec` frame carries
  the terminal's nonce, which no output can know. A frame without it still makes a block, since
  pills and bars show what a frame claims and send nothing. Remote terminals get no nonce.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the pointer is over a block whose first row is on screen, the terminal shall show Copy beside its pill, and Rerun when the shell is at a prompt | driven |
| REQ-002 | WHEN Copy is clicked, the block's output shall be on the clipboard | driven |
| REQ-003 | WHEN Rerun is clicked, the terminal shall be sent Ctrl-U, the command and a carriage return | driven |
| REQ-004 | WHILE a block runs, no Rerun shall be offered | driven |
| REQ-005 | WHERE a block's `preexec` frame did not carry the terminal's nonce, the terminal shall offer no Rerun for it | driven |
| REQ-006 | WHEN Marley's bash or zsh integration reports a command, its frame shall carry the terminal's nonce, which no program the shell starts inherits, and a remote terminal shall be given none | PTY |
| REQ-007 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec. **P2 Code** — the element, the buttons, the ledger rows first.
  **P3 Test** — the driven tests, negative checks, a capture, the gate. **P4 Complete** —
  docs, ledger, close, archive, commit.
