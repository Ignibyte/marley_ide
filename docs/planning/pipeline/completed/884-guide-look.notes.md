# The Marley Guide in Marley's colours — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-884-search-branches-history-extensions-guide.md
- **Pipeline spec:** 884-guide-look.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #883; split while planning: the guide's look here, the
  rest of the SearchMore board TICKET-891.
- **Classification / tier:** feature, S. A page Marley's crate carries.
- **Recall (§18.3):** #599 (the guide is one HTML page shown in a Browser tab).
- **Discovery:** a probe of the branch picker (`script/e2e/891-branch-picker.sh`, kept as the
  probe for #891): Zed's already groups local and remote branches, ticks the current one and gives
  each its last commit; the board adds coloured dots and notes. The guide already has its nav,
  find field and articles, in a generic palette.
- **Decisions:** D1 in the spec.

### Design
- `guide/index.html`: the `:root` variables from Day's values, the dark block from Night's, a
  `--card` for the article card (`article`: card background, border, radius, padding).
- **File manifest:** `guide/index.html`; the scenario.

## Phase 2 — Code
- **Built:** `guide/index.html`: the `:root` variables take Marley Day's values (background, panel,
  card, text, muted, line, the accent #a94a0c, code, the "What it is" box) and the dark block
  Marley Night's (the accent #ffb27a); `article` is a card (its background, border, 12 px radius,
  padding). No text changed.
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/884-guide-look.sh` (sway): `marley: open guide`; the run's Chromium
  starts light, so a stand-in agent sets the scheme with `browser_color_scheme` (#756).
- `884-01-guide`: the dark scheme: Night's background and panel, the nav with its filter, "What
  Marley adds to Zed" as a card with its "What it is" box. REQ-001 met.
- `884-02-day`: Marley Day and the light scheme: Day's cream and white card. REQ-001 met.
- The first run, before the scheme was set, showed the light scheme under Marley Night: the page
  follows the browser's scheme, not Marley's theme (D1).


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation c22e7bbb.
- Ticket closed; pipeline archived; TICKET-891 holds the rest of the board, its branch probe
  `script/e2e/891-branch-picker.sh`.
