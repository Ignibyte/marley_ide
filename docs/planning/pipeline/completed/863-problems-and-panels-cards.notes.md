# The problems and panels cards above the bottom row — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-863-cards-above-the-bottom-row-and-the-debugging-row.md
- **Pipeline spec:** 863-problems-and-panels-cards.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row after #862.
- **Classification / tier:** feature, M. One small Zed hunk, the panel group.
- **Recall (§18.3):** a probe of the row: Zed's language servers and edit predictions buttons
  already open cards; the problems count opens the drawer; nothing lists the panels. The
  debugging row needs a debug adapter in the run, and Ask an agent is new behaviour: split to
  #901. The brain had nothing.
- **Decisions:** D1, D2 in the spec.

## Phase 2 — Code
- `diagnostics/src/items.rs`: `MarleyProblemsCard(fn(&App) -> bool)`; while it answers yes the
  count is a `PopoverMenu` whose `ContextMenu` card holds Problems, Errors N, Warnings N and Open
  in the drawer (`Deploy`'s path, warnings shown when there are no errors, as Zed's click does).
- `panel_group.rs`: `init` sets the card's global to `marley_layout`; the group ends with a
  chevron whose card lists every panel but Debug in the board's order, "name · where" (D2:
  `panel_tabs::is_moved`, Files beside the rail, else the dock), each row the panel's toggle with
  its key.
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/863-problems-and-panels-cards.sh` (sway, rust-analyzer).
- `863-01-problems`: the count's card above it: Problems, Errors 1, Warnings 0, Open in the drawer
  with Ctrl-Shift-M (REQ-001).
- `863-02-drawer`: after Open in the drawer, the Problems drawer with `missing`'s error
  (REQ-001).
- `863-03-panels`: the Panels card: Files · beside the rail (Ctrl-Shift-E), Changes · tab,
  Outline · tab, Knowledge · tab, Fleet · tab, Containers · tab, Problems · bottom drawer, Collab
  · tab, Agent · right dock (Ctrl-?), Terminal · bottom drawer (REQ-002).
- `863-04-day`: the same card in Marley Day (REQ-002).
- The first runs: the tooltip of the count's button covered the card's last row (the pointer
  moves off now), and the group's chevron sat at 1578, not 1590.

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, the ledger row, look-plan W1.4.
- Brain decision recorded on consultation 96cc50c8.
