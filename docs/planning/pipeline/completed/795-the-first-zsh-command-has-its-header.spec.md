---
pipeline_id: 2e7e10d2-b5bd-4dce-a502-d2e54b9515eb
ticket: docs/planning/tickets/open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md
status: Phase 4 — Complete PASS
title: The first command in a new zsh on a Mac is a block under its header
type: bug
slice: The block terminal on macOS (#792)
references: []
---

## Title
#792's check on the Mac (2026-10-16) saw the first command in a new zsh draw without Marley's
block header, the second with one. Linux never shows it (2026-10-29). This checks it on the Mac
with the current build, and fixes what the check finds.

## Scope
### In
- The current build installed on the Mac (`just install --remote-server-from dev`).
- A new zsh terminal there and its first and second commands, typed and sent by an agent.
- A fix, if the check sees the header missing, in the block terminal or the zsh integration.

### Out (explicitly deferred)
- #949 (a shell given with arguments gets no integration).

## Reference (§20)
Warp's blocks (`docs/warp_architecture/`): every command, the first included, is a block with its
header; Marley's block terminal follows that since #463 and #465. Zed's `terminal` crate draws the
grid; the header is Marley's.

### Prior art
- `marley_terminal/shell_integration/marley.zsh` (#465): the hooks install at the first
  `precmd`, which prints `init`, `bootstrapped` and `precmd` before the first prompt; a PTY run
  of the Mac's startup on Linux showed that order, the same for the first and second commands.
- #792 (`-l` on macOS, the shell as Marley's own child) and F-claude-792.
- #764 (restored blocks): a restored terminal's first new command is a live block on Linux.

## UI proof
- On the Mac, `screencapture` through iTerm (its Screen Recording grant): `795-mac-first`, a new
  zsh terminal after two typed commands, both blocks under their headers (REQ-001).
- On Linux, `script/e2e/795-the-first-zsh-command-has-its-header.sh`: `795-01-fresh` and
  `795-02-restored` (REQ-002).

## Locked-In Decisions
- D1 — The Mac's own build decides: what #792 saw was on the build of 2026-10-16, and Linux cannot
  show macOS's zsh or PTY.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the first command is typed in a new zsh terminal on a Mac, Marley shall draw it as a block under its header, as it does the second. | `795-mac-first` |
| REQ-002 | WHEN the first command is typed in a new zsh terminal on Linux, fresh or restored, Marley shall draw it as a block under its header. | `795-01-fresh`, `795-02-restored` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the fix the Mac check calls for, or none; `script/gates.sh --diff` green.
- **P3 Test** — the Mac shot and the Linux scenario, every shot read.
- **P4 Complete** — CHANGELOG, docs, knowledge, close the ticket, archive, commit.
