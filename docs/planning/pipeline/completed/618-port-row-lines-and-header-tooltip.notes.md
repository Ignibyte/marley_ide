# A port row's clipped lines, and a header tooltip over its menu — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-618-port-row-lines-and-header-tooltip.md
- **Pipeline spec:** 618-port-row-lines-and-header-tooltip.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - F-claude-603-a-long-trailing-state-squeezed-the-rows-text-out-001: a row line's state is for a few short words.
  - #603's Test phase: the URL and unit clip at the default width while the hover buttons keep their room; `visible_on_hover` keeps its layout.
  - #606's Test phase: a header tooltip shown at the right-click stays over the menu until the pointer moves; gpui clears only a visible tooltip on mouse-down.
  - Zed withholds a trigger's tooltip while its menu is open (`dock.rs`, `PopoverMenu::trigger_with_tooltip`).
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted** from `queued/`; the seams re-verified. Since /spec, #615 gave the port row its
  right-click menu with the tooltip built in the trigger behind `!menu_open`, and moved the unit
  onto a line of its own with its state; #617 added the closed project's port rows. Line numbers:
  `render_port_row` 5380, `port_row_buttons` 5309, `row_card` 7732, `RowLine` 7706, the closed
  header's tooltip 4301 and its menu 4312.
- **Recall (§18.3):** the queued notes' four bullets stand. The brain (`rusty-cli brain ask`, consultation
  ed64aa1c22f34fee99ed3751903c5a5d): nothing on this seam.
- **Prior art:** as the spec cites; `Hsla::blend` added for the overlay's shade.

### Approach
- **The overlay (D1):** `port_row_buttons` loses its place in the row's flex: it becomes an
  `absolute()` strip at the row's right, `inset_y_0`, its buttons centered, on an opaque shade:
  `panel_background.blend(ghost_element_hover)`, or `.blend(ghost_element_selected)` for the
  selected row, since the buttons show only while the pointer is on the row. The row gets
  `relative()`. The text then has the row's width, and the strip covers its end only on hover.
- **The URL line:** `url_label(url)` drops `http://` before `127.0.0.1:` and `localhost:` for the
  line only; Open and Copy keep the whole URL. `RowLine` gains `cut: Cut` (`End`, `Start`,
  `Middle`), applied with `Label::truncate`, `truncate_start` or `truncate_middle`; the URL line
  cuts in the middle, the unit line at its start, every other line at its end as now.
- **The tooltip:** the port row's tooltip starts with its full URL.
- **The header:** the closed header's tooltip moves into the trigger, behind `!menu_open`.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_workbench/src/rail.rs` | Marley | the overlay, the lines' cuts, the URL label, the tooltips |
| `script/e2e/618-port-row-lines-and-header-tooltip.sh` | e2e | the scenario |

No Zed crate changes.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | a user unit with a long name serving in the scratch project (#603's pattern), at the default rail width | `row.png`: host, port and the unit's end readable; `tooltip.png`: the tooltip starting with the URL |
| REQ-002 | the pointer on the row | `hover.png`: Open, Copy, Stop over the row's end, the text in place |
| REQ-003 | a second project handed over, `repo` shown, Marley started again so the second is closed; the pointer on its header until the tooltip shows, then a right-click | `header-tooltip.png`, `menu.png`: the menu with no tooltip over it |

### Risks and decisions
- The overlay's shade assumes the rail draws on `panel_background`, which it does (`rail.rs` root);
  a theme whose hover color is opaque gets the same color, which is right.

## Phase 2 — Code (2026-09-30)
- **Built (`rail.rs`):** `port_row_buttons` is an absolute strip at the row's right on
  `panel_background.blend(ghost_element_hover)` (`ghost_element_selected` for the selected row),
  shown on hover; the row is `relative()`. `RowLine.cut` (`Cut::End`, `Start`, `Middle`) picks the
  label's truncation in `row_card`; the URL line is `url_label` (no `http://` for 127.0.0.1 and
  localhost) cut in the middle, the unit line cut at its start. The port row's tooltip starts with
  the whole URL. The closed header's tooltip moved into the right-click trigger, behind
  `!menu_open`. `render_port_row`'s doc comment had landed on `port_row_buttons`; each has its own
  now.
- **Deviations from the plan:** none.
- **Review:** no entity is read or updated in render beyond the theme; Open and Copy still use the
  whole URL; every other row's lines keep `Cut::End`.
- **Clippy found:** `inset_y_0` is not a gpui style method (`top_0` and `bottom_0` instead).
- **The scenario** `script/e2e/618-port-row-lines-and-header-tooltip.sh` is written ahead of Test
  so one gate run covers it.
- **Gate:** `just gate-diff` GREEN, 17 PASS.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/618-port-row-lines-and-header-tooltip.sh` (sway): a user unit
  `marley-e2e-618-a-service-with-a-long-name-<port>` serving `repo`; `repo-b` handed over; `repo`
  shown and Marley started again with no path. One run, every shot read.
- **Shots:**
  - `row.png` (REQ-001): the port row at the default width reads `:46715 python3`, then
    `127.0.0.1:46715/` whole, then `…vice-with-a-long-name-46715.service`, cut at its start. The
    container rows' URLs read `127.0.0.1:8081/` and `127.0.0.1:8083/` whole too.
  - `hover.png` (REQ-002): the pointer on the row; Open, Copy and Stop sit over its end on the
    row's hover shade, covering the unit line's end; the title and the URL stay where they were.
  - `tooltip.png` (REQ-001): the row's tooltip begins `http://127.0.0.1:46715/`, then the command,
    the folder, the pid, the user service, and the double-click hint.
  - `restart.png`: `repo-b` closed (dimmed, no rows, no chevron), `repo` open.
  - `header-tooltip.png`: the pointer on `repo-b`'s header shows "Not open. Click to open it."
  - `menu.png` (REQ-003): after the right-click, the pointer unmoved, the menu (Move Project Up
    disabled, Move Project Down, No Archived Threads, Clear Browser Data… disabled, Remove
    Project) and no tooltip over it.
- **No fix needed;** the gate's run already covers the scenario.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Fixed); `workbench-shell.md`; `marley_workbench.md` (the Port
  rows bullet); the guide and the walkthrough (2.x's port row check). The guide page has no port
  row detail to change.
- **Knowledge:** L-claude-618-hover-buttons-over-a-rows-end-need-an-opaque-shade-001. No `F-`
  block: the bugs fixed here were filed as Test notes of #603 and #606.
- **Brain:** consultation ed64aa1c22f34fee99ed3751903c5a5d closed with `brain decide`.

