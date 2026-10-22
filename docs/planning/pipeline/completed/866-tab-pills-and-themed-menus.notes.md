# Tabs with their kind and state, the + menu's keys, and the split menu drawn — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-866-tabs-as-pills-and-themed-menus.md
- **Pipeline spec:** 866-tab-pills-and-themed-menus.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row after #865.
- **Classification / tier:** feature, M. Two small hooks in `workspace::pane`; Marley files.
- **Recall (§18.3):** a probe shot the tab bar and both menus; #883's tiles; #870 made prompts
  cards in the theme, so the tmux close prompt is one already. The brain had nothing.
- **Decisions:** D1 in the spec.

## Phase 2 — Code
- `pane.rs` (Zed): `MarleyTabLook`/`MarleyTabMark` (the tile in the icon's place, the dot after
  the title); `MarleySplitPictures` and `marley_split_rows` (custom rows: picture, direction, key,
  each dispatching its split to the pane's focus).
- `tab_tiles.rs`: `glyph_of` and `tile`, shared with the switcher; `tab_mark` (Marley layout only),
  `state_dot` (D1: the last block running or failed, or the terminal unread), `split_picture` (a
  22×16 frame with the new pane's side in the accent).
- `rail.rs`: New Terminal and New Agent name `workspace::NewTerminal` and `marley::NewAgent`, so
  the + menu shows their keys.
- Changed after the first shot: the left and right pictures drew no panes, since a row centres
  its children and gave them no height; each pane fills its share now.
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/866-tab-pills-and-themed-menus.sh` (sway).
- `866-01-tabs`: "repo — bash" led by ">_" with the error's dot (its `false` failed), "main.rs"
  led by "rs" (REQ-001).
- `866-02-plus`: New Terminal with Ctrl-~, New Agent… with Ctrl-Alt-N (REQ-002).
- `866-03-split`: Right, Left, Up and Down, each with its picture (the new pane's side lit) and
  Ctrl-K and its arrow (REQ-003).
- `866-04-day`: the tab bar in Marley Day (REQ-001).
- Seen: a terminal tab still shows its own icon in its title after the tile, as `TerminalView`'s
  tab content draws it.

## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, the ledger row, look-plan W2.1.
- Brain decision recorded on consultation 52246918.
