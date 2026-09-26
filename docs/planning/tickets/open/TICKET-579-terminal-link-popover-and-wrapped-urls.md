# TICKET-579 — A popover on a terminal link, the default asked once, and wrapped URLs joined

- **Ticket:** LOCAL #579 (feature, prong 3 with prong 1, slice 2 of #503's routing, B6b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet; `/pipeline:plan` mints the pair
- **Source ticket:** TICKET-503's Out (slice 2, minted at its Complete, 2026-09-26); the Orca survey's `LinkActionPopover` and the two wrapped-link files (report 03 §2.3)
- **Status:** open

## Summary
#503 routes a Ctrl+clicked URL and an OSC 8 link's plain click, and offers a dev server's URL in
the footer. This slice adds three things:
- **The popover.** A plain click on a URL in the grid opens a small popover: Open in Browser
  Tab, Open in System Browser, and Copy Link (for an OSC 8 link, the hidden target). This is
  Orca's `LinkActionPopover`, and what Warp's plain-click tooltip and right-click menu do.
- **The default, asked once.** The first click asks which place should be the default, and the
  answer is written to `marley.terminal_links`.
- **Wrapped URLs.** A URL a program wrapped at the right edge, or drew inside a box frame (`│ ┃
  ║`), is joined across its rows, so it opens whole and its hover covers every row.

Zed's side:
- a public link-at-a-point on `Terminal` (`find_hyperlink_at_point` is private today);
- a plain-click path in `mouse_up` that opens the popover when the press and the release land
  on one link with no drag;
- the joining in `alacritty/hyperlinks.rs` beside `find_from_grid_point`, with the hover range
  across the rows.

Each needs its row in `docs/marley/zed-touchpoints.md`. Orca's two joining files are 331 lines,
and its popover 197. #503's notes (Out, and the split) have the detail.

## Acceptance
- A plain click on a URL in a terminal shows the popover with its three entries.
- The first click asks which place is the default, and the answer is kept in
  `marley.terminal_links`.
- A URL wrapped at the edge or drawn in a box opens whole from any of its rows.
- #503's scenario still passes.
