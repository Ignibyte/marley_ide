# Needs you entries as cards — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-840-needs-you-toasts-and-prompts.md
- **Pipeline spec:** 840-needs-you-cards.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #839; Chad's "lets continue on". Split while planning:
  toasts and prompts became TICKET-869 (Zed's notification frame, a dozen prompt surfaces).
- **Classification / tier:** feature, S. Marley crate only (`rail.rs`).
- **Recall (§18.3):** #833's stand-in scenario; L-claude-836. The brain: nothing on this seam.
- **Discovery:** `render_inbox` (rail.rs:2384), `render_inbox_entry` (:2423), `row_card`,
  `InboxAnswer`; the inbox's order (marley_rail.rs:638).
- **Decisions:** D1, D2 in the spec.

### Design
- `render_inbox`: the heading row (NEEDS YOU, a count pill, "longest first" at the end).
- `render_inbox_entry`: a card (`border_1`, `rounded_md`, `elevated_surface_background`): a
  `h_flex` of the tile (icon on `element_background`, 24 px) and a `v_flex` of who · where with
  waited at the end, the ask (two lines at most); then the chips and buttons as today, the buttons'
  style by `answer_weight(answer)`: Filled + accent tint, Outlined, Subtle.
- **File manifest:** `crates/marley_workbench/src/rail.rs`, the scenario.

### Visual check plan
As the spec's UI proof.

## Phase 2 — Code
- **Built:** the heading (NEEDS YOU, a pill, "longest first"); `render_inbox_entry` split into
  `inbox_icon`, `render_inbox_card` and `render_inbox_buttons` (too many lines otherwise), the
  card framed; `answer_style`; Open for an entry no answer reaches in place.
- **Deviation:** the board's Open button did not exist (the card's click was the only way); added,
  as a subtle button doing what the click does.
- **Gate:** `just gate-diff` green (17 gates and the receipt).


## Phase 3 — Test
- Scenario `script/e2e/840-needs-you-cards.sh` (sway), the gated build: #833's stand-in in b, then
  in c 65 s later.
- `840-01-night`: NEEDS YOU, a "2" pill, "longest first"; b's card (Claude Code · b, 1 m,
  "Permission for Write: README.md", Open) over c's (now) (REQ-001, REQ-002).
- `840-02-day`: the same in Marley Day: white cards on cream (REQ-001, REQ-002).
- The filled and outlined weights need an answer in place (a Zed thread's tool call or Codex);
  no scenario raises one here: read in the diff (`answer_style`).

## Phase 4 — Complete
- **Docs:** CHANGELOG, the guide's Needs you, the architecture note's inbox bullet, the look plan.
- **Knowledge:** none new (the card follows #827's row card and #836's lessons).
- **Brain:** decision on consultation 6627aa36 (the planning ask, e0db5975, had lapsed).
- Ticket closed, pipeline archived; TICKET-869 filed for toasts and prompts.
