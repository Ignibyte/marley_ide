---
pipeline_id: 7d84dab3-026c-49eb-870e-69eadeb745cf
ticket: docs/planning/tickets/open/TICKET-476-prompt-at-the-bottom.md
status: Phase 4 — Complete PASS
title: The prompt at the bottom of the terminal
type: feature
slice: prong 1 T1d
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/470-blocks-drawn-in-the-terminal.spec.md]
---

## Title
Draw a terminal's content against the bottom edge while its screen has room, so the prompt sits
on the last row, as Warp pins its input to the bottom.

## Scope
### In
- `Content` reports the last used line of the live screen: the lowest line holding the cursor or
  a character other than a space (`crates/terminal/src/alacritty.rs`, `make_content`).
- A pure `marley_terminal::bottom_shift`: the empty rows below that line, less the rows the view
  is scrolled back by, and none on the alternate screen.
- `TerminalElement::prepaint` moves the grid's origin down by that many rows once the content is
  synced, and hands the moved bounds to `Terminal::set_size`, so painting, the mouse and the
  block decorations share one origin. The sub-row padding goes to the top while it applies.
  Only for a standalone terminal view, like Zed's own bottom anchoring.

### Out (explicitly deferred)
- Marley's own command editor docked at the bottom (T3).
- Warp's other input positions ("start at the top" is Zed's today) as a setting.

## Reference (§20)
- **Warp:** "Pin to the bottom" (Warp mode), its default input position: input pinned to the
  bottom of the terminal view, blocks flowing up and out of view
  (https://docs.warp.dev/terminal/appearance/input-position/), also seen in Chad's session on
  2026-09-23. No Warp code.
- **Upstream Zed:** `TerminalElement::prepaint` already anchors the sub-row padding to the
  bottom when the bottom row is occupied (`should_anchor_to_bottom`); this extends it to whole
  rows.

### Prior art
- **Behavior maps:** `docs/warp_architecture/` has nothing on the input's position; the
  published docs above are the reference.
- **Published material:** none needed; no escape sequence changes.
- **Code we already ship:** `Content::bottom_row_occupied` and `scrolled_to_bottom` scan the
  same cells in `make_content`. `Terminal::set_size` stores the bounds' origin at once and
  resizes the PTY only when the lines, columns or cell size change, so moving the origin sends
  no SIGWINCH, and the mouse maps through the stored bounds. Claude Code uses synchronized
  updates (mode 2026), so its redraws are not drawn half done.

## UI proof
UI-AFFECTING: where a terminal's rows are drawn, and what a click hits.
- **Driven tests** (`terminal_view`): over a real PTY, two short blocks draw their pills on the
  grid's last rows; with mouse reporting on, a click on the text reports its grid row; scrolled
  back by one line, the pills stay where they were and the history shows above them; on the
  alternate screen the content starts at the top.
- **Live drive:** `just shot` of a new terminal: the prompt on the last row.

## Locked-In Decisions
- D1 — A drawing change only: the grid, the PTY's size, the scrollback and the block anchors
  stay as they are.
- D2 — The shift is computed from the content after `sync`, in the same frame, so new output is
  never drawn a frame late below the pane.
- D3 — Scrolled back by d rows, the shift is d rows smaller, so the history appears above the
  content instead of the content jumping.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the live screen has empty rows below its last used line, the terminal shall draw that line on the pane's last row | driven |
| REQ-002 | WHILE the view is scrolled back by d rows, the shift shall be d rows smaller, and never below 0 | unit + driven |
| REQ-003 | WHILE the alternate screen shows, the terminal shall draw from the top as before | unit + driven |
| REQ-004 | WHEN the user clicks or selects in shifted content, the terminal shall act on the row drawn there | driven |
| REQ-005 | The block decorations (bars, pills, buttons) shall be drawn on their shifted rows | driven |
| REQ-006 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; promotion re-verifies the seams and asks the brain.
- **P2 Code** — the `Content` field, the pure shift, the prepaint hunk; the ledger rows first.
- **P3 Test** — the driven and unit tests, negative checks, a capture, the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
