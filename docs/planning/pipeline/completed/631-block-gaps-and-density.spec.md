---
pipeline_id: ec67031e-b6f7-45d5-b0af-d61eeafa11eb
ticket: docs/planning/tickets/closed/TICKET-631-block-gaps-and-density.md
status: Phase 4 — Complete PASS
title: "Gaps between blocks and a header density setting"
type: feature
slice: prong 1 T5
references: [docs/marley/three-prong-plan.md]
---

## Title
Stage two's last slice (plan T5, D3): blocks stand apart and a one-row prompt's header shows two
lines, behind `marley.block_density` (`comfortable`, the default, or `compact`, today's look).
#628 to #630 drew a header only in the rows its prompt took; this slice draws space that is no
grid line, through a display-row map in Zed's terminal element and its inverse in the mouse paths.
Chad picked it on 2026-10-01 ("lets do 637, 631, 540"), which settles the ticket's open question:
the look is worth the diff in Zed's terminal crates.

## Scope
### In
- `marley_terminal::RowMap` (pure): the pixels inserted above viewport rows (a gap above each
  block that starts on screen, the live prompt's included, and a header's extra line above a
  one-row prompt), the anchor (the live screen keeps its last row on the bottom edge; a view
  scrolled back keeps its top row on the top edge), the offset of a row, and the inverse from a
  pixel back to the grid.
- `terminal_view::TerminalElement`: the map built in `prepaint` from the spans and the header
  hook's rows, and every row-to-y site routed through it: text runs, block glyphs and backgrounds
  (by row at paint), the cursor and the IME bounds, selection and search ranges (split where a
  gap falls inside), the block elements, headers, the last-row chip, the wash, gutter and outlines
  (`marley_rows_bounds`, `marley_gutter_bounds`), and `marley_block_at`.
- `terminal::Terminal`: the element's map stored with the size; the eight places a window position
  becomes a grid position (`marley_link_at`, `mouse_move`, `select_word_at_event_position`,
  `mouse_drag`, `mouse_down`, `mouse_up`, `scroll_wheel`'s report, the hyperlink lookup) take it
  through the map.
- `MarleyBlockSpacing`, a global the workbench sets from `marley.block_density`: the gap in rows
  and whether a one-row prompt's header takes a second line. The header hook gets the rows the
  header covers, one more than the prompt's in that case, so #630's two-line layout draws.
- `marley.block_density` in `MarleySettingsContent`, `default.json` and the Marley page.

### Out (explicitly deferred)
- Pixel-smooth scrolling: the view still scrolls by grid lines, so the anchor changes between the
  live screen and a view scrolled back one line moves the rows by the space inserted.
- The pinned header (#529) stays one row in either density.
- The inline (embedded) terminal, a terminal with a block below its cursor (Inline Assist), and
  the alternate screen: no map, rows as Zed draws them.
- Block navigation landing a block in the rows the live screen hides at its top (the live screen
  shows its newest rows; scrolled back, a block lands on the top edge as before).

## Reference (§20)
- **Warp (behavior):** a block list whose blocks stand apart, each with a header above its
  command (`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6, lines 237-247:
  `BlockList` holds "filtering, scroll position, gaps", the Block's `header_grid`). The tree has
  no note that sizes Warp's density; half a row and a two-line header are Marley's own (#630's
  spec said so).
- **Upstream Zed:** `terminal_view`'s `TerminalElement` (its `prepaint` and `paint`, the
  `scroll_top` it already offsets by pixels for `block_below_cursor`) and `terminal`'s mouse
  mapping (`grid_point_and_side`, `content_index_for_mouse`), kept: the map is applied around
  them, never inside the alacritty grid.

### Prior art
- **Behavior maps:** the plan's D3 ("a display-row map in `TerminalElement` like the editor's block
  map"); AD-claude-470, AD-claude-476 (the bottom shift, the content moved by its origin, which
  the mouse maps through), AD-claude-529, AD-claude-628 (the thirteen row-to-y and four pixel-to-row
  sites); #629's display-row navigation.
- **Published material:** none beyond Warp's docs on blocks.
- **Code we already ship:** Zed's editor `BlockMap` (display rows that are no buffer line) is the
  model, but the terminal element has no `DisplaySnapshot`, and the terminal scrolls by grid
  lines, so the map here is a list of inserts over viewport rows. The element's existing
  `scroll_top` and the bottom shift show the two ways it already moves rows by pixels: an offset
  at paint, undone on the mouse path. No crate owns a terminal display-row map.

## UI proof
`script/e2e/631-block-gaps-and-density.sh`, `compositor sway` (clicks, a drag, the wheel): bash
with `PS1='$ '` in a git repository, the density at its default. Shots: `631-01-comfortable`,
`631-02-word`, `631-03-drag`, `631-04-menu`, `631-05-compact`, `631-06-full`, `631-07-scrolled`,
`631-08-back`.

## Locked-In Decisions
- D1 — `comfortable` (half-row gaps, two-line headers for one-row prompts) is the default: T5 is
  the Warp look; `compact` keeps #630's.
- D2 — The live screen is anchored at the bottom (the live prompt stays on the last row; the
  inserted space pushes the oldest rows up past the top edge), a view scrolled back at the top
  (a block navigated to lands on the top edge, the pinned header covers row 0; the newest rows of
  that view pass the bottom edge). Every row is on screen at one of the two.
- D3 — The map lives in the element and is handed to the terminal with its size; the terminal
  turns a window position into a grid position through it at the one place each mouse path
  subtracts the origin, so the internal events (selection updates, hyperlink lookups, vi motion)
  stay in grid space.
- D4 — A gap or a header's extra line belongs to the row below it: a click there lands on that
  row's first pixel.
- D5 — No map on the alternate screen, in an inline terminal, or with a block below the cursor.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the density is comfortable, each block that starts on screen shall stand half a row below the one above it | shot `631-01-comfortable` |
| REQ-002 | WHILE the density is comfortable and the headers are on, a block whose prompt took one row shall show a two-line header, the folder and branch over the command | shot `631-01-comfortable` |
| REQ-003 | WHILE the view is at the live screen, the shell's prompt shall stay on the last row | shots `631-01-comfortable`, `631-06-full` |
| REQ-004 | WHEN a word in the output of a block below a gap is double-clicked, the selection shall cover that word | shot `631-02-word` |
| REQ-005 | WHEN a drag selects rows of two blocks, the highlight shall lie on the selected rows, broken at the gap | shot `631-03-drag` |
| REQ-006 | WHEN a block's output row below a gap is right-clicked, that block shall be the one the menu acts on | shot `631-04-menu` (the block's outline) |
| REQ-007 | WHEN the view is scrolled back, its rows shall start at the top edge, and scrolling to the live screen shall bring the prompt back to the last row | shots `631-07-scrolled`, `631-08-back` |
| REQ-008 | WHERE the density is compact, blocks shall have no gaps and a one-row prompt's header shall take one row | shot `631-05-compact` |
| REQ-009 | WHILE a full-screen program shows, or a block sits below the cursor, the rows shall be drawn as Zed draws them | review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the touchpoint rows first; `RowMap`; the element's sites; the terminal's mouse
  paths; the density global and setting; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
