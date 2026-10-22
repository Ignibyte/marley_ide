# The focused pane outlined in the accent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-853-splits-and-ports.md
- **Pipeline spec:** 853-focused-pane.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #852; split while planning: the outline here, the
  switcher, ports, containers and Collab TICKET-883.
- **Classification / tier:** feature, S. Two Zed paths (the defaults file, `pane_group.rs`) and
  Marley's theme file.
- **Recall (§18.3):** `pane_group.rs` already carries Marley's row shares (#798);
  `default.json`'s row lists every Marley default.
- **Discovery:** Zed paints the active pane's inset border when `border_size` is set, in
  `border_selected`, which Marley Night has at a dim #7a4420; `pane.focused_border` is read only by
  the debugger.
- **Decisions:** D1, D2 in the spec.

### Design
- Ledger rows first (both Zed paths), then the default, the one-line colour change with its
  `// Marley:` comment, and the theme's two values (#ff8c42, #b4500e).
- **File manifest:** `assets/settings/default.json` (Zed), `crates/workspace/src/pane_group.rs`
  (Zed), `assets/themes/marley/marley.json` (Marley's file in Zed's tree), the ledger, the
  scenario.

## Phase 2 — Code
- **Built:** the ledger rows for `pane_group.rs` and `default.json` first; `border_size: 2.0` with
  its `Marley:` comment; `PaneAxisElement::paint` paints the border in `pane_focused_border`
  (a `// Marley:` hunk of one line); `pane.focused_border` #ff8c42 in Night, #b4500e in Day.
- **Gate:** green on the first run, 17 of 17.
- **Review:** Zed draws the border only for a leaf pane in a split, so a single pane has none;
  the debugger's focused sub-pane, the one other reader of the colour, takes the accent too.


## Phase 3 — Test
- **Scenario:** `script/e2e/853-focused-pane.sh` (sway): `rail.rs` opened, `pane: split right`,
  then `workspace: activate pane left`.
- `853-01-split`: the right pane outlined in #ff8c42, the left none. REQ-001 met.
- `853-02-other`: the outline on the left pane, none on the right. REQ-001 met.
- `853-03-day`: Marley Day, the left pane outlined in the deep orange. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed); the ledger rows checked against what shipped; look-plan W5 row. No Marley
  crate changed, so no per-crate note.
- Knowledge: none new. Brain: `brain decide` on consultation 51dc9072.
- Ticket closed; pipeline archived; TICKET-883 holds the rest of the board.
