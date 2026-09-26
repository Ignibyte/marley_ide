---
pipeline_id: b9a9a62b-6fb1-4e89-9edc-604806fe04e2
ticket: docs/planning/tickets/open/TICKET-546-block-reads-under-the-alternate-screen.md
status: Phase 4 — Complete PASS
title: "Block reads while a full-screen program shows"
type: bug
slice: prong 1 T0 (the block terminal) with prong 2's terminal tools
references: [docs/planning/pipeline/completed/544-blocks-survive-a-rewrap.notes.md, docs/planning/pipeline/completed/491-marley-mcp.spec.md]
---

## Title
While a full-screen program (vim, less, htop) holds the alternate screen, an agent's
`terminal_read` of an earlier block answers the block's output from the main screen, where the
blocks live, and `terminal_blocks` says from the main screen whether that output is still kept.

## Scope
### In
- `crates/terminal/src/alacritty.rs` (Zed crate, the Marley function `absolute_lines_text`): the
  line arithmetic and the text read from the main screen's grid.
- `crates/terminal/src/terminal.rs` (Zed crate, the Marley method `block_output_kept`): the main
  screen's evicted count.
- The vendored alacritty (`vendor/alacritty_terminal/src/term/mod.rs`):
  `Term::main_bounds_to_string`, `bounds_to_string` over `main_grid()` (#544), added beside it.

### Out (explicitly deferred)
- Drawing blocks over the alternate screen: the view draws none there, by design (#470).

## Reference (§20)
N/A — Marley-specific: Marley's block reads over alacritty's two screens. alacritty keeps the main
screen in `inactive_grid` while the alternate one shows (`Term::swap_alt`), and its own
`bounds_to_string` reads the active grid, which is what a selection on the screen wants.

### Prior art
- **The code we already ship.** `Term::main_grid` (#544); `Term::bounds_to_string` and its private
  `line_to_string` (vendored alacritty), which read `self.grid`; Zed's selection and search, which
  read the active screen on purpose; `absolute_lines_text` and `block_output_kept` (#464, #491).
  No crate we build reads the inactive screen's text today.
- **Published material:** none needed.

## UI proof
UI-AFFECTING (what agents read). `script/e2e/546-block-reads-under-the-alternate-screen.sh`
(`compositor sway`): `seq 1 3` runs, then `less` opens a file and holds the alternate screen; the
stand-in agent reads `seq 1 3`'s block before, while `less` shows (`546-01-less-open`), and after it
quits; the reads must be equal, and `terminal_blocks` must call the output kept while `less` shows.

## Locked-In Decisions
- D1 — Read the main screen, never the alternate one: blocks are the main screen's.
- D2 — The vendored addition copies `bounds_to_string`'s loop over a given grid rather than
  reshaping upstream's function, so a re-sync re-adds it whole.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a full-screen program holds the alternate screen, `terminal_read` of a finished block shall answer the output it answered before the program started. | The run log (reads before, during and after `less`) and shot `546-01-less-open` |
| REQ-002 | WHILE a full-screen program holds the alternate screen, `terminal_blocks` shall report a finished block's output as kept when the main screen still holds it. | The run log |

## Phase Plan
- **P1 Plan** — this spec; the design in the notes.
- **P2 Code** — `main_bounds_to_string`, the two readers.
- **P3 Test** — the scenario (red on the unfixed build first); the golden set; the gate.
- **P4 Complete** — docs, knowledge, close, archive, commit.
