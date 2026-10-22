# The needs-you pulse — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-858-motion-and-focus.md
- **Pipeline spec:** 858-needs-you-pulse.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next W6 row after #857.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #827's state dots; gpui's animation honours `reduce_motion`.
- **Decisions:** D1 in the spec.

### Design
- `rail.rs` `row_card`: for `RowState::NeedsYou`, the dot `with_animation` keyed by the row's
  icon selector: over 1.2 s, `bump = |sin(2π·t)|`, its size 10 + 2·bump px and a shadow ring
  of 3·bump px in the dot's colour; at t = 1 the bump is 0, the dot at rest.
- **File manifest:** `rail.rs`; the scenario.

## Phase 2 — Code
- **Built:** `rail.rs`: `pulse(dot, id, color)` animates a needs-you dot over 1.2 s (`bump =
  |sin(2π·t)|`, size `2·bump + 10` px by `mul_add`, a shadow ring of its colour), keyed by the
  row's icon selector; `row_card` uses it for `RowState::NeedsYou`. The gpui imports gain
  `Animation`, `AnimationExt`, `BoxShadow` and `point`, which made `rail_tests.rs`'s
  `gpui::point` a redundant path (clippy), now `point`.
- **Gate:** green, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/858-needs-you-pulse.sh` (sway), #827's setup (#542's stand-in Claude
  Code on a private bus); `b` worked, then waited on the user. #728's stand-in was tried first: its
  plan ask reaches Needs you but its row stays idle, so the dot never changed.
- `858-01-pulse-a` to `-d` (every 0.25 s from the step): b's dot larger with a pink ring in each.
  `858-02-still` (3 s on): the dot at rest, no ring. REQ-001 met.
- `858-03-reduced-a` to `-d` (`REDUCED=1`, `reduce_motion` on): the dot at rest from the first.
  REQ-002 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W6 line.
- Knowledge: none new. Brain: `brain decide` on consultation 27a1d4aa.
- Ticket closed; pipeline archived; TICKET-895 holds the rail's shrink and the focus ring.
