---
pipeline_id: 1ee6d33e-f9b4-41bd-aa7a-0ccec34cad66
ticket: docs/planning/tickets/closed/TICKET-595-drive-bar-outlives-its-program.md
status: Phase 4 — Complete PASS
title: End the drive bar with the program an agent typed into
type: bug
slice: T-series, agent terminal writes (#525's follow-up)
references: [docs/planning/pipeline/completed/525-agent-drives-a-running-program.spec.md]
---

## Title
The bar under a terminal that says what an agent typed, with Take Over, shows only while the
program it typed into is still in the foreground. Today it stays under the shell after that
program exits.

## Scope
### In
- `terminal_drive::footer`: the bar's arm checks the drive's program against the terminal's
  foreground program.

### Out (explicitly deferred)
- The pending card: a write that waits resolves itself, refused once the program changed.
- "You have control" and Hand Back: a take-over lasts until the user hands back (#525 REQ-006).
- Ctrl-I: `toggle_control` already reads the drive through `drive()`, which drops the last write
  when the program changes.

## Reference (§20)
N/A — Marley-specific: an agent typing into a terminal's program is Marley's (#525); neither
Zed's terminal nor a reference product shows this bar.

### Prior art
- The code we ship: `terminal_drive::drive()` already compares the stored program with
  `foreground_program` and resets the drive when they differ; the footer, which gets `&App`,
  can make the same comparison without the reset.
- Behavior maps: nothing on this seam.

## UI proof
The scenario `script/e2e/525-agent-types-into-a-terminal.sh` (`compositor sway`):
- `525-11-shell-refused`: after Ctrl-D leaves Python, no bar under the shell.
- `525-11b-tab-completes`: `ech` and Ctrl-I at the shell's prompt complete to `echo`, #525's
  REQ-012 seen.

## Locked-In Decisions
- D1 — The bar needs the drive's program to be the foreground program; nothing else changes in
  the footer.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the program an agent typed into leaves the foreground, the terminal shall show no drive bar. | Shot 525-11-shell-refused |
| REQ-002 | WHILE that program still runs, the bar shall show as before. | Shots 525-03, 525-10 |
| REQ-003 | WHEN the user presses Ctrl-I at the shell's prompt after that program exits, the key shall reach the shell. | Shot 525-11b-tab-completes |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the check in `footer`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — #525's scenario with the Tab step; every shot read.
- **P4 Complete** — CHANGELOG, the crate note, the corrections to #594's and #525's notes, an
  `F-` block, close, archive, commit.
